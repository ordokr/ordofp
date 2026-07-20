//! WGSL shader generation for `ParFlumen` operations.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use alloc::string::String;

/// A closed set of GPU reduce operators.
///
/// The public string API (`"+"`, `"*"`, `"min"`, `"max"`) is parsed into this
/// enum before any shader is generated; unknown operators never reach WGSL.
///
/// # Latin Etymology
/// *Operatio reductionis* = operation of reduction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatioReductionis {
    /// `+`
    Additio,
    /// `*`
    Multiplicatio,
    /// `min`
    Minimum,
    /// `max`
    Maximum,
}

impl OperatioReductionis {
    /// Parse the public operator string. Returns `None` for anything not in
    /// the closed set — callers turn that into a `GpuError`, never a shader.
    #[inline]
    pub fn parse(op: &str) -> Option<Self> {
        match op {
            "+" => Some(Self::Additio),
            "*" => Some(Self::Multiplicatio),
            "min" => Some(Self::Minimum),
            "max" => Some(Self::Maximum),
            _ => None,
        }
    }

    /// The identity element for out-of-bounds lanes, as a WGSL literal.
    #[inline]
    pub fn identity(self, type_name: &str) -> &'static str {
        match (self, type_name) {
            (Self::Additio, "f32") => "0.0",
            (Self::Additio, _) => "0",
            (Self::Multiplicatio, "f32") => "1.0",
            (Self::Multiplicatio, _) => "1",
            // WGSL has no f32::MAX constant; spell the literal.
            (Self::Minimum, "f32") => "3.4028235e38",
            (Self::Minimum, "i32") => "2147483647",
            (Self::Minimum, _) => "4294967295u",
            (Self::Maximum, "f32") => "-3.4028235e38",
            // `-2147483648` alone overflows WGSL's abstract-int negation.
            (Self::Maximum, "i32") => "-2147483647 - 1",
            (Self::Maximum, _) => "0u",
        }
    }

    /// Emit the combining expression for two WGSL operands.
    #[inline]
    pub fn combine(self, a: &str, b: &str) -> alloc::string::String {
        use alloc::format;
        match self {
            Self::Additio => format!("{a} + {b}"),
            Self::Multiplicatio => format!("{a} * {b}"),
            Self::Minimum => format!("min({a}, {b})"),
            Self::Maximum => format!("max({a}, {b})"),
        }
    }
}

/// Generate a WGSL compute shader for elementwise map operation.
///
/// # Arguments
///
/// * `entry_point` - Shader entry point name (e.g., "map")
/// * `operation` - WGSL expression for the operation (e.g., "x * 2.0")
/// * `workgroup_size` - Workgroup size (default: 64, should be multiple of 64)
/// * `type_name` - WGSL type name (e.g., "f32", "i32", "u32")
///
/// # Example
///
/// ```rust
/// use ordofp_core::par::codegen::wgsl::generate_map_shader;
///
/// let shader = generate_map_shader("double", "x * 2.0", 64, "f32");
/// assert!(shader.contains("double"));
/// assert!(shader.contains("x * 2.0"));
/// ```
#[inline]
pub fn generate_map_shader(
    entry_point: &str,
    operation: &str,
    workgroup_size: u32,
    type_name: &str,
) -> String {
    // Ideal workgroup size depends on hardware, but should generally be a multiple of 64
    // Common sizes: 64x1x1, 256x1x1
    // Pre-allocate: map shader template is ~300 chars plus small variable substitutions.
    // Using write! into a pre-allocated String avoids the intermediate String created
    // inside alloc::format! before it is returned.
    let mut out =
        String::with_capacity(320 + entry_point.len() + operation.len() + type_name.len());
    // Rust 1.88+: Use r"..." instead of r#"..."# when no escaping needed
    use core::fmt::Write as _;
    let _ = write!(
        out,
        r"// Input buffer
@group(0) @binding(0)
var<storage, read> input: array<{type_name}>;

// Output buffer
@group(0) @binding(1)
var<storage, read_write> output: array<{type_name}>;

struct Params {{
    element_count: u32,
}}

@group(0) @binding(2)
var<uniform> params: Params;

@compute @workgroup_size({workgroup_size})
fn {entry_point}(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;

    // Bounds check against the LOGICAL element count. The storage buffer's
    // raw byte capacity may exceed the live data (first-fit pool reuse) —
    // never bound on that physical capacity.
    if (index >= params.element_count) {{
        return;
    }}

    // Apply operation
    let x = input[index];
    output[index] = {operation};
}}
"
    );
    out
}

/// Generate a WGSL compute shader for reduction operation.
///
/// # Arguments
///
/// * `entry_point` - Shader entry point name (e.g., "`reduce_sum`")
/// * `op` - The reduce operator (already parsed from its public string form)
/// * `workgroup_size` - Workgroup size (default: 64)
/// * `type_name` - WGSL type name (e.g., "f32", "i32", "u32")
///
/// # Note
///
/// Reduction requires multiple passes for large arrays. This generates
/// a single-pass reduction shader that works within a workgroup.
#[inline]
pub fn generate_reduce_shader(
    entry_point: &str,
    op: OperatioReductionis,
    workgroup_size: u32,
    type_name: &str,
) -> String {
    let identity = op.identity(type_name);
    let combine = op.combine(
        "shared_data[local_index]",
        "shared_data[local_index + offset]",
    );

    // Pre-allocate: reduce shader template is ~550 chars plus variable substitutions.
    let mut out = String::with_capacity(580 + entry_point.len() + combine.len() + type_name.len());
    // Rust 1.88+: Use r"..." instead of r#"..."# when no escaping needed
    use core::fmt::Write as _;
    let _ = write!(
        out,
        r"// Input buffer
@group(0) @binding(0)
var<storage, read> input: array<{type_name}>;

// Output buffer (reduced value)
@group(0) @binding(1)
var<storage, read_write> output: array<{type_name}>;

struct Params {{
    element_count: u32,
}}

@group(0) @binding(2)
var<uniform> params: Params;

// Shared memory for workgroup reduction
var<workgroup> shared_data: array<{type_name}, {workgroup_size}>;

@compute @workgroup_size({workgroup_size})
fn {entry_point}(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_index) local_index: u32) {{
    let index = global_id.x;

    // Load data into shared memory, bounded by the LOGICAL element count.
    // The storage buffer's raw byte capacity may exceed the live data
    // (first-fit pool reuse) — never bound on that physical capacity.
    if (index < params.element_count) {{
        shared_data[local_index] = input[index];
    }} else {{
        shared_data[local_index] = {identity};
    }}

    workgroupBarrier();

    // Tree reduction
    var offset = {workgroup_size}u / 2u;
    while (offset > 0u) {{
        if (local_index < offset) {{
            shared_data[local_index] = {combine};
        }}
        workgroupBarrier();
        offset = offset / 2u;
    }}

    // Write result
    if (local_index == 0u) {{
        // Write to output at the workgroup index
        let workgroup_id = global_id.x / {workgroup_size}u;
        output[workgroup_id] = shared_data[0];
    }}
}}
"
    );
    out
}

/// Kernel cache key for shader pipelines.
///
/// This is used to cache compiled shader pipelines to avoid recompilation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KernelKey {
    /// Shader source code (used as cache key).
    pub shader_source: String,
    /// Entry point name.
    pub entry_point: String,
}

impl KernelKey {
    /// Create a new kernel key.
    #[inline]
    pub fn new(shader_source: String, entry_point: String) -> Self {
        Self {
            shader_source,
            entry_point,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_shader_bounds_on_params_not_array_length() {
        let s = generate_reduce_shader("reduce", OperatioReductionis::Additio, 64, "f32");
        assert!(
            s.contains("var<uniform> params: Params"),
            "missing params uniform:\n{s}"
        );
        assert!(
            s.contains("index < params.element_count"),
            "must bound on logical count:\n{s}"
        );
        assert!(
            !s.contains("arrayLength"),
            "physical buffer length must not gate loads:\n{s}"
        );
    }

    #[test]
    fn reduce_identity_matches_operator() {
        let mul = generate_reduce_shader("reduce", OperatioReductionis::Multiplicatio, 64, "f32");
        assert!(
            mul.contains("= 1.0;"),
            "f32 product identity must be 1.0:\n{mul}"
        );
        let min_u = generate_reduce_shader("reduce", OperatioReductionis::Minimum, 64, "u32");
        assert!(
            min_u.contains("= 4294967295u;"),
            "u32 min identity must be u32 max:\n{min_u}"
        );
        let max_i = generate_reduce_shader("reduce", OperatioReductionis::Maximum, 64, "i32");
        assert!(
            max_i.contains("= -2147483647 - 1;"),
            "i32 max identity must be i32::MIN:\n{max_i}"
        );
    }

    #[test]
    fn min_max_emit_call_syntax_not_infix() {
        let s = generate_reduce_shader("reduce", OperatioReductionis::Minimum, 64, "f32");
        assert!(
            s.contains("min(shared_data[local_index], shared_data[local_index + offset])"),
            "{s}"
        );
        assert!(
            !s.contains("] min shared_data"),
            "infix `a min b` is invalid WGSL:\n{s}"
        );
    }

    #[test]
    fn parse_rejects_unknown_ops() {
        assert!(OperatioReductionis::parse("+").is_some());
        assert!(OperatioReductionis::parse("min").is_some());
        assert!(OperatioReductionis::parse("^").is_none());
        assert!(OperatioReductionis::parse("").is_none());
    }

    #[test]
    fn map_shader_bounds_on_params() {
        let s = generate_map_shader("map", "x * 2.0", 64, "f32");
        assert!(s.contains("var<uniform> params: Params"), "{s}");
        assert!(s.contains("index >= params.element_count"), "{s}");
        assert!(!s.contains("arrayLength"), "{s}");
    }
}
