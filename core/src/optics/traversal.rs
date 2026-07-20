use crate::optics::{Aspectus, Divisio};
use crate::typeclasses::Applicatio;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Iteratio (Traversal) - Focus on zero or more elements.
///
/// > *"Iteratio est actus repetendi."*
/// > — Iteration is the act of repeating.
///
/// A Traversal allows accessing and modifying multiple elements of a structure,
/// possibly with effects (using Applicatio).
pub trait Iteratio<S, A> {
    /// Modify the focused elements effectfully.
    fn traverse<F: Applicatio<Inner = A>>(&self, s: S, f: impl Fn(A) -> F) -> F::Target<S>
    where
        F::Target<S>: Applicatio<Inner = S>;

    /// Modify the focused elements using a pure function.
    ///
    /// Required (no default): a panicking default body would turn a missing
    /// override into a runtime trap instead of a compile error.
    fn modify(&self, s: S, f: impl Fn(A) -> A) -> S;

    /// Collect all focused elements.
    #[cfg(feature = "alloc")]
    fn collect(&self, s: &S) -> Vec<A>;
}

impl<S, A, GetFn, SetFn> Iteratio<S, A> for Aspectus<S, A, GetFn, SetFn>
where
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    #[inline]
    fn traverse<F: Applicatio<Inner = A>>(&self, s: S, f: impl Fn(A) -> F) -> F::Target<S>
    where
        F::Target<S>: Applicatio<Inner = S>,
    {
        let a = self.get(&s);
        let fb = f(a);
        fb.map(move |b| self.set(&s, b))
    }

    #[inline]
    fn modify(&self, s: S, f: impl Fn(A) -> A) -> S {
        self.modify(&s, f)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn collect(&self, s: &S) -> Vec<A> {
        use alloc::vec;
        vec![self.get(s)]
    }
}

impl<S, A, PreviewFn, ReviewFn> Iteratio<S, A> for Divisio<S, A, PreviewFn, ReviewFn>
where
    S: Clone,
    PreviewFn: Fn(&S) -> Option<A>,
    ReviewFn: Fn(A) -> S,
{
    #[inline]
    fn traverse<F: Applicatio<Inner = A>>(&self, s: S, f: impl Fn(A) -> F) -> F::Target<S>
    where
        F::Target<S>: Applicatio<Inner = S>,
    {
        if let Some(a) = self.preview(&s) {
            let fb = f(a);
            fb.map(move |b| self.review(b))
        } else {
            F::pure_target(s)
        }
    }

    #[inline]
    fn modify(&self, s: S, f: impl Fn(A) -> A) -> S {
        if let Some(val) = self.modify(&s, f) {
            val
        } else {
            s
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn collect(&self, s: &S) -> Vec<A> {
        self.preview(s).into_iter().collect()
    }
}

// Composition of traversals requires higher-kinded bounds on F::Target,
// which is not currently available in a clean way in the GAT-based Applicatio
// trait, so traversal composition is deliberately not provided.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optics::{aspectus, divisio};
    use alloc::string::{String, ToString};
    use alloc::vec;

    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    #[test]
    fn test_aspectus_iteratio_modify() {
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let modified = name_lens.modify(&person, |n: String| n.to_uppercase());
        assert_eq!(modified.name, "ALICE");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_aspectus_iteratio_collect() {
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name| Person { name, age: p.age },
        );

        let person = Person {
            name: "Bob".to_string(),
            age: 25,
        };

        let collected = name_lens.collect(&person);
        assert_eq!(collected, vec!["Bob".to_string()]);
    }

    #[test]
    fn test_divisio_iteratio_modify_some() {
        let some_prism = divisio(|opt: &Option<i32>| *opt, Some);

        let value = Some(42);
        let modified = some_prism.modify(&value, |x| x * 2);
        assert_eq!(modified, Some(Some(84)));
    }

    #[test]
    fn test_divisio_iteratio_modify_none() {
        let some_prism = divisio(|opt: &Option<i32>| *opt, Some);

        let value: Option<i32> = None;
        let modified = some_prism.modify(&value, |x| x * 2);
        assert_eq!(modified, None);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_divisio_iteratio_collect_some() {
        let some_prism = divisio(|opt: &Option<i32>| *opt, Some);

        let value = Some(42);
        let collected = some_prism.collect(&value);
        assert_eq!(collected, vec![42]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_divisio_iteratio_collect_none() {
        let some_prism = divisio(|opt: &Option<i32>| *opt, Some);

        let value: Option<i32> = None;
        let collected = some_prism.collect(&value);
        assert!(collected.is_empty());
    }
}
