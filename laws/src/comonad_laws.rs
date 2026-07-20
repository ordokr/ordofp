//! # Comonad Laws
//!
//! This module provides property-based laws for testing [`Comonad`] implementations.
//!
//! ## Laws
//!
//! 1. **Left Identity**: `w.extend(extract) == w`
//! 2. **Right Identity**: `w.extend(f).extract() == f(w)`
//! 3. **Associativity**: `w.extend(f).extend(g) == w.extend(|v| g(v.extend(f)))`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::comonad_laws;
//! use ordofp::comonad::{Comonad, Identitas};
//!
//! // Test left identity law for Identitas
//! assert!(comonad_laws::identitas_left_identity(Identitas(42)));
//!
//! // Test right identity law
//! assert!(comonad_laws::identitas_right_identity(Identitas(42), |w| w.extract() * 2));
//! ```

use crate::is_eq::IsEq;
use ordofp::comonad::{Comonad, Contextus, Identitas};

// ==================== Identitas Laws ====================

/// **Left Identity Law** for Identitas: `w.extend(extract) == w`
pub fn identitas_left_identity<A: Clone + Eq>(w: Identitas<A>) -> bool {
    let extended = w.extend(ordofp::comonad::Comonad::extract);
    extended.extract() == w.extract()
}

/// **Right Identity Law** for Identitas: `w.extend(f).extract() == f(w)`
pub fn identitas_right_identity<A: Clone + Eq, B: Clone + Eq, F>(w: Identitas<A>, f: F) -> bool
where
    F: Fn(&Identitas<A>) -> B,
{
    w.extend(&f).extract() == f(&w)
}

/// **Associativity Law** for Identitas: `w.extend(f).extend(g) == w.extend(|v| g(v.extend(f)))`
pub fn identitas_associativity<A, B, C, F, G>(w: Identitas<A>, f: F, g: G) -> bool
where
    A: Clone,
    B: Clone + Eq,
    C: Clone + Eq,
    F: Fn(&Identitas<A>) -> B + Clone,
    G: Fn(&Identitas<B>) -> C + Clone,
{
    let lhs = w.extend(f.clone()).extend(g.clone());
    let rhs = w.extend(|v: &Identitas<A>| g(&v.extend(f.clone())));
    lhs.extract() == rhs.extract()
}

/// Returns an [`IsEq`] for the Identitas left identity law.
pub fn identitas_left_identity_eq<A: Clone>(w: Identitas<A>) -> IsEq<A> {
    let extended = w.extend(ordofp::comonad::Comonad::extract);
    IsEq::equal_under_law(extended.extract(), w.extract())
}

// ==================== Contextus Laws ====================

/// **Left Identity Law** for Contextus: `w.extend(extract) == w`
pub fn contextus_left_identity<E: Clone + Eq, A: Clone + Eq>(w: Contextus<E, A>) -> bool {
    let extended = w.extend(ordofp::comonad::Comonad::extract);
    extended.extract() == w.extract() && *extended.ask() == *w.ask()
}

/// **Right Identity Law** for Contextus: `w.extend(f).extract() == f(w)`
pub fn contextus_right_identity<E: Clone + Eq, A: Clone + Eq, B: Clone + Eq, F>(
    w: Contextus<E, A>,
    f: F,
) -> bool
where
    F: Fn(&Contextus<E, A>) -> B,
{
    w.extend(&f).extract() == f(&w)
}

/// **Associativity Law** for Contextus: `w.extend(f).extend(g) == w.extend(|v| g(v.extend(f)))`
pub fn contextus_associativity<E, A, B, C, F, G>(w: Contextus<E, A>, f: F, g: G) -> bool
where
    E: Clone + Eq,
    A: Clone,
    B: Clone + Eq,
    C: Clone + Eq,
    F: Fn(&Contextus<E, A>) -> B + Clone,
    G: Fn(&Contextus<E, B>) -> C + Clone,
{
    let lhs = w.extend(f.clone()).extend(g.clone());
    let rhs = w.extend(|v: &Contextus<E, A>| g(&v.extend(f.clone())));
    lhs.extract() == rhs.extract()
}

/// Returns an [`IsEq`] for the Contextus left identity law.
pub fn contextus_left_identity_eq<E: Clone, A: Clone>(w: Contextus<E, A>) -> IsEq<A> {
    let extended = w.extend(ordofp::comonad::Comonad::extract);
    IsEq::equal_under_law(extended.extract(), w.extract())
}

/// **Environment preservation**: extend preserves the environment
pub fn contextus_preserves_environment<E: Clone + Eq, A: Clone, B, F>(
    w: Contextus<E, A>,
    f: F,
) -> bool
where
    F: Fn(&Contextus<E, A>) -> B,
{
    let extended = w.extend(f);
    *extended.ask() == *w.ask()
}

// ==================== Duplicate Laws ====================

/// **Duplicate/Extend relationship**: `w.duplicate() == w.extend(|x| x.clone())`
pub fn identitas_duplicate_extend_relationship<A: Clone + Eq>(w: Identitas<A>) -> bool {
    let duplicated = w.duplicate();
    let extended = w.extend(std::clone::Clone::clone);
    duplicated.extract().extract() == extended.extract().extract()
}

/// **Extract/Duplicate relationship**: `w.duplicate().extract() == w`
pub fn identitas_extract_duplicate_relationship<A: Clone + Eq>(w: Identitas<A>) -> bool {
    w.duplicate().extract().extract() == w.extract()
}

/// **Duplicate/Duplicate relationship**: `w.duplicate().duplicate()` maps structure correctly
pub fn identitas_duplicate_duplicate<A: Clone + Eq>(w: Identitas<A>) -> bool {
    // The outermost value should be the original
    let dup_dup = w.duplicate().duplicate();
    dup_dup.extract().extract().extract() == w.extract()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Identitas Tests ====================

    #[test]
    fn test_identitas_left_identity_law() {
        fn test(w: i32) -> bool {
            identitas_left_identity(Identitas(w))
        }
        quickcheck(test as fn(i32) -> bool);
    }

    #[test]
    fn test_identitas_right_identity_law() {
        fn test(w: i8) -> bool {
            identitas_right_identity(Identitas(w), |x| x.extract().wrapping_mul(2))
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_identitas_associativity_law() {
        fn test(w: i8) -> bool {
            identitas_associativity(
                Identitas(w),
                |x: &Identitas<i8>| x.extract().wrapping_add(1),
                |x: &Identitas<i8>| x.extract().wrapping_mul(2),
            )
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_identitas_duplicate_extend() {
        fn test(w: i32) -> bool {
            identitas_duplicate_extend_relationship(Identitas(w))
        }
        quickcheck(test as fn(i32) -> bool);
    }

    #[test]
    fn test_identitas_extract_duplicate() {
        fn test(w: i32) -> bool {
            identitas_extract_duplicate_relationship(Identitas(w))
        }
        quickcheck(test as fn(i32) -> bool);
    }

    #[test]
    fn test_identitas_duplicate_duplicate() {
        fn test(w: i32) -> bool {
            identitas_duplicate_duplicate(Identitas(w))
        }
        quickcheck(test as fn(i32) -> bool);
    }

    // ==================== Contextus Tests ====================

    #[test]
    fn test_contextus_left_identity_law() {
        fn test(env: i32, val: i32) -> bool {
            contextus_left_identity(Contextus::new(env, val))
        }
        quickcheck(test as fn(i32, i32) -> bool);
    }

    #[test]
    fn test_contextus_right_identity_law() {
        fn test(env: i8, val: i8) -> bool {
            contextus_right_identity(Contextus::new(env, val), |w| {
                w.extract().wrapping_add(*w.ask())
            })
        }
        quickcheck(test as fn(i8, i8) -> bool);
    }

    #[test]
    fn test_contextus_associativity_law() {
        fn test(env: i8, val: i8) -> bool {
            contextus_associativity(
                Contextus::new(env, val),
                |w: &Contextus<i8, i8>| w.extract().wrapping_add(*w.ask()),
                |w: &Contextus<i8, i8>| w.extract().wrapping_mul(2),
            )
        }
        quickcheck(test as fn(i8, i8) -> bool);
    }

    #[test]
    fn test_contextus_preserves_environment() {
        fn test(env: i8, val: i8) -> bool {
            contextus_preserves_environment(Contextus::new(env, val), |w| {
                w.extract().wrapping_mul(2)
            })
        }
        quickcheck(test as fn(i8, i8) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_identitas_tests() {
        // Left identity
        assert!(identitas_left_identity(Identitas(42)));
        assert!(identitas_left_identity(Identitas("hello")));

        // Right identity
        assert!(identitas_right_identity(Identitas(10), |w| w.extract() * 3));
        assert!(identitas_right_identity(Identitas(5), |w| w.extract() + 10));

        // Associativity
        assert!(identitas_associativity(
            Identitas(5),
            |w: &Identitas<i32>| w.extract() + 1,
            |w: &Identitas<i32>| w.extract() * 2,
        ));
    }

    #[test]
    fn manual_contextus_tests() {
        let w = Contextus::new("config", 42);

        // Left identity
        let extended = w.extend(ordofp::comonad::Comonad::extract);
        assert_eq!(extended.extract(), w.extract());

        // Environment preservation
        assert!(contextus_preserves_environment(
            Contextus::new("config", 42),
            |x| x.extract() * 2
        ));

        // Using environment in computation
        let result = w.extend(|x| format!("{}: {}", x.ask(), x.extract()));
        assert_eq!(result.extract(), "config: 42");
    }

    #[test]
    fn test_duplicate_laws() {
        let w = Identitas(42);

        // duplicate().extract() == w
        assert_eq!(w.duplicate().extract().extract(), w.extract());

        // cmap equivalence
        let mapped = w.cmap(|x| x * 2);
        assert_eq!(mapped.extract(), 84);
    }

    #[test]
    fn test_identitas_eq() {
        let eq = identitas_left_identity_eq(Identitas(42));
        assert!(eq.holds());

        let eq = contextus_left_identity_eq(Contextus::new("env", 42));
        assert!(eq.holds());
    }
}
