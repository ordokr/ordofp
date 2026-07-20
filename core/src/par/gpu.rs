//! GPU-specific Nodus nodes and types for the `ParFlumen` pipeline.

use alloc::sync::Arc;
use alloc::vec::Vec;

use super::Nodus;
use super::backend;

/// GPU operation descriptor for WGSL shader generation.
///
/// This describes a chain of operations that can be compiled to a single GPU kernel.
#[derive(Clone, Debug)]
pub struct GpuMapChain {
    /// Source data.
    pub source_len: usize,
    /// WGSL expression to apply (uses 'x' as input variable).
    pub operations: alloc::vec::Vec<alloc::string::String>,
}

impl GpuMapChain {
    /// Create a new GPU map chain.
    #[inline]
    pub fn new(source_len: usize) -> Self {
        Self {
            source_len,
            // Typical kernel fuse depth is shallow; 4 avoids the first realloc.
            operations: alloc::vec::Vec::with_capacity(4),
        }
    }

    /// Add an operation to the chain.
    #[inline]
    pub fn push_op(&mut self, wgsl_expr: alloc::string::String) {
        self.operations.push(wgsl_expr);
    }

    /// Compose all operations into a single WGSL expression.
    #[inline]
    pub fn compose_wgsl(&self) -> alloc::string::String {
        if self.operations.is_empty() {
            return alloc::string::String::from("x");
        }
        // Chain operations: each operation's output becomes the next input
        let mut expr = alloc::string::String::from("x");
        for op in &self.operations {
            // Replace the standalone identifier 'x' in the operation with
            // the current expression
            expr = replace_ident_x(op, &alloc::format!("({expr})"));
        }
        expr
    }
}

/// True if `b` can be part of a WGSL identifier (`[A-Za-z0-9_]`).
#[inline]
fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Replace every *standalone* identifier `x` in `op` with `replacement`.
///
/// An `x` is standalone only when neither neighbouring byte is an identifier
/// byte, so `x` inside longer identifiers such as `exp(x)` or `max(a, x)` is
/// left untouched (a naive `str::replace("x", ..)` would mangle the function
/// names themselves).
fn replace_ident_x(op: &str, replacement: &str) -> alloc::string::String {
    let bytes = op.as_bytes();
    let mut out = alloc::string::String::with_capacity(op.len() + replacement.len());
    let mut copied_to = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'x'
            && (i == 0 || !is_ident_byte(bytes[i - 1]))
            && (i + 1 == bytes.len() || !is_ident_byte(bytes[i + 1]))
        {
            out.push_str(&op[copied_to..i]);
            out.push_str(replacement);
            copied_to = i + 1;
        }
    }
    out.push_str(&op[copied_to..]);
    out
}

// =============================================================================
// NodusGpuMap - GPU-capable map with WGSL expression
// =============================================================================

pub(crate) struct NodusGpuMap<T> {
    /// Previous node.
    pub(crate) prev: Arc<dyn Nodus<Item = T>>,
    /// WGSL expression (uses 'x' as input variable).
    pub(crate) wgsl_expr: alloc::string::String,
    /// Rust fallback function.
    pub(crate) fallback: Arc<dyn Fn(T) -> T + Send + Sync>,
}

impl<T: backend::wgpu::GpuScalar + Clone> Nodus for NodusGpuMap<T> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let f = &self.fallback;
        self.prev.visit_scalar_ref(&mut |a| sink(f(*a)));
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        let f = &self.fallback;
        self.prev.visit_scalar_ref(&mut |a| {
            let b = f(*a);
            sink(&b);
        });
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        let a = self.prev.get(index);
        (self.fallback)(a)
    }

    #[inline]
    fn try_gpu_map_chain(&self) -> Option<GpuMapChain> {
        // Try to get chain from previous node
        let mut chain = self.prev.try_gpu_map_chain()?;
        // Add our operation
        chain.push_op(self.wgsl_expr.clone());
        Some(chain)
    }

    #[inline]
    fn try_as_gpu_source(&self) -> Option<(&[u8], &'static str)> {
        self.prev.try_as_gpu_source()
    }
}

// =============================================================================
// NodusInitGpu - Source node for Pod data with GPU support
// =============================================================================

pub(crate) struct NodusInitGpu<T> {
    pub(crate) data: Vec<T>,
}

impl<T: backend::wgpu::GpuScalar + Clone> Nodus for NodusInitGpu<T> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        for item in &self.data {
            sink(*item);
        }
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        for item in &self.data {
            sink(item);
        }
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        true
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        self.data[index]
    }

    #[inline]
    fn try_gpu_map_chain(&self) -> Option<GpuMapChain> {
        // Source node starts a new chain with identity operation
        Some(GpuMapChain::new(self.data.len()))
    }

    #[inline]
    fn try_as_gpu_source(&self) -> Option<(&[u8], &'static str)> {
        Some((bytemuck::cast_slice(&self.data), T::wgsl_type()))
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;
        self.data.par_iter().cloned().collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn chain_of(ops: &[&str]) -> GpuMapChain {
        let mut chain = GpuMapChain::new(0);
        for op in ops {
            chain.push_op(String::from(*op));
        }
        chain
    }

    #[test]
    fn test_compose_wgsl_exp_not_mangled() {
        let chain = chain_of(&["x * 2.0", "exp(x)"]);
        let composed = chain.compose_wgsl();
        assert_eq!(composed, "exp(((x) * 2.0))");
        // The function name itself must survive: no `e(...)p(...)` mangling.
        assert!(
            composed.contains("exp("),
            "function name mangled: {composed}"
        );
    }

    #[test]
    fn test_compose_wgsl_max_survives() {
        let chain = chain_of(&["max(x, 1.0)"]);
        assert_eq!(chain.compose_wgsl(), "max((x), 1.0)");
    }

    #[test]
    fn test_compose_wgsl_multi_op_nesting() {
        // Mirrors the documented simple ops (`sin(x)`, `x * 2.0`): chaining
        // still nests each op's output as the next op's input.
        let chain = chain_of(&["sin(x)", "x * 2.0"]);
        assert_eq!(chain.compose_wgsl(), "(sin((x))) * 2.0");
    }
}
