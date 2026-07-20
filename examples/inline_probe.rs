//! Cross-crate inlining probe.
//!
//! Compiled as part of the `ordofp` crate, this links against the separately
//! compiled `ordofp_core` rlib — a real downstream crate boundary, exactly as
//! a downstream consumer sees it. The `#[unsafe(no_mangle)] #[inline(never)]`
//! wrappers give stable symbols to disassemble; the question is whether each
//! wrapper's body contains the monomorphized AVL walk inlined, or a `call` into
//! a separately-compiled `ordofp_core` `get`/`get_node` symbol (a missed
//! optimization that would only be recovered by fat LTO).
//!
//! Build (stock-release-like, no LTO — the common consumer profile):
//!   cargo rustc --release --example `inline_probe` -- -C lto=off -C codegen-units=16 --emit asm

use ordofp_core::pfds::{OrdMap, OrdSet};
use std::hint::black_box;

/// A UI consumer's `ImmutableFormState::get_field` pattern: `OrdMap`<String,_> queried
/// by &str. If `get` inlines across the boundary, this body is a tree-walk loop
/// with no call into `ordofp_core`.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn probe_get<'a>(m: &'a OrdMap<String, String>, k: &str) -> Option<&'a String> {
    m.get(k)
}

/// `ImmutableFormState::is_touched`/`has_errors` pattern.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn probe_contains(m: &OrdMap<String, String>, k: &str) -> bool {
    m.contains_key(k)
}

/// `OrdSet`<String> membership by &str (UI-style tag/flag sets).
#[unsafe(no_mangle)]
#[inline(never)]
pub fn probe_set_contains(s: &OrdSet<String>, k: &str) -> bool {
    s.contains(k)
}

fn main() {
    let mut m = OrdMap::new();
    let mut s = OrdSet::new();
    for i in 0..32 {
        m = m.insert(format!("field_{i:03}"), format!("value_{i}"));
        s = s.insert(format!("tag_{i:03}"));
    }
    // Keep the probes (and their inputs) live so codegen can't fold them away.
    black_box(probe_get(black_box(&m), black_box("field_016")));
    black_box(probe_contains(black_box(&m), black_box("field_016")));
    black_box(probe_set_contains(black_box(&s), black_box("tag_016")));
}
