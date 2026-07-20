//! # Fixpoint Laws
//!
//! Laws relating to [`Fix`], `cata` (catamorphism), and `ana` (anamorphism).

use ordofp::fix::Fix;
use ordofp::typeclasses::hkt::FunctorHKT;

/// **Lambek's Lemma 1**: `unfix(new(x)) == x`
///
/// This property asserts that wrapping a value in `Fix` and then unwrapping it yields the original value.
pub fn lambek_lemma_1<F>(x: F::Target<Fix<F>>) -> bool
where
    F: FunctorHKT,
    F::Target<Fix<F>>: PartialEq + Clone,
{
    let fix_x = Fix::<F>::new(x.clone());
    let unfix_x = fix_x.unfix();
    unfix_x == x
}

/// **Lambek's Lemma 2**: `new(unfix(x)) == x`
///
/// This property asserts that unwrapping a `Fix` value and then wrapping it again yields the original value.
pub fn lambek_lemma_2<F>(x: Fix<F>) -> bool
where
    F: FunctorHKT,
    F::Target<Fix<F>>: Clone,
    Fix<F>: PartialEq + Clone,
{
    let unfix_x = x.clone().unfix();
    let new_x = Fix::<F>::new(unfix_x);
    new_x == x
}

/// **Cata-Ana Inverse Property**:
///
/// If `alg` and `coalg` are inverses, then `cata . ana = id`.
///
/// `cata(ana(a))` should be `a`.
pub fn cata_ana_inverse<F, A, Alg, Coalg>(a: A, mut alg: Alg, mut coalg: Coalg) -> bool
where
    F: FunctorHKT,
    A: PartialEq + Clone,
    Alg: FnMut(F::Target<A>) -> A,
    Coalg: FnMut(A) -> F::Target<A>,
{
    let fixed = Fix::<F>::ana(a.clone(), &mut coalg);
    let unfixed = fixed.cata(&mut alg);
    unfixed == a
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordofp::typeclasses::hkt::HKT;
    use quickcheck::quickcheck;

    // Define a FunctorHKT to test with.
    // We use a simple structure isomorphic to Option:
    // NatF A = Zero | Succ A
    #[derive(Clone, Debug, PartialEq)]
    pub enum NatF<A> {
        Zero,
        Succ(A),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct NatHKT;

    impl HKT for NatHKT {
        type Target<T> = NatF<T>;
    }

    impl FunctorHKT for NatHKT {
        fn map<A, B, F>(fa: NatF<A>, mut f: F) -> NatF<B>
        where
            F: FnMut(A) -> B,
        {
            match fa {
                NatF::Zero => NatF::Zero,
                NatF::Succ(a) => NatF::Succ(f(a)),
            }
        }
    }

    #[test]
    fn test_cata_ana_inverse() {
        // Use Nat <-> u32 isomorphism

        // Use small numbers to avoid stack overflow in recursion
        fn prop(n: u8) -> bool {
            let coalg = |n: u32| -> NatF<u32> {
                if n == 0 {
                    NatF::Zero
                } else {
                    NatF::Succ(n - 1)
                }
            };

            let alg = |nf: NatF<u32>| -> u32 {
                match nf {
                    NatF::Zero => 0,
                    NatF::Succ(n) => n + 1,
                }
            };

            cata_ana_inverse::<NatHKT, _, _, _>(u32::from(n), alg, coalg)
        }

        quickcheck(prop as fn(u8) -> bool);
    }
}
