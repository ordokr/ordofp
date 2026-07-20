extern crate quickcheck;
use ordofp::transformers::{EitherT, OptionT, ReaderT, StateT};
use quickcheck::quickcheck;

// ============================================================================
// OptionT Laws
// ============================================================================

// OptionT<Result<Option<A>, E>>
// Left Identity: pure(a).flat_map(f) == f(a)
fn optiont_left_identity(a: i8, f_seed: i8) -> bool {
    let f = move |x: i8| -> OptionT<Result<Option<i8>, String>> {
        if x.wrapping_add(f_seed) % 2 == 0 {
            OptionT::some(x)
        } else {
            OptionT::none()
        }
    };

    let lhs = OptionT::<Result<Option<i8>, String>>::some(a).flat_map(f);
    let rhs = f(a);
    lhs.run() == rhs.run()
}

// Right Identity: m.flat_map(pure) == m
fn optiont_right_identity(a: Option<i8>) -> bool {
    let m = if let Some(v) = a {
        OptionT::<Result<Option<i8>, String>>::some(v)
    } else {
        OptionT::<Result<Option<i8>, String>>::none()
    };

    let lhs = m.clone().flat_map(OptionT::some);
    let rhs = m;
    lhs.run() == rhs.run()
}

// Associativity: m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
fn optiont_associativity(a: Option<i8>, f_seed: i8, g_seed: i8) -> bool {
    let m = if let Some(v) = a {
        OptionT::<Result<Option<i8>, String>>::some(v)
    } else {
        OptionT::<Result<Option<i8>, String>>::none()
    };

    let f = move |x: i8| -> OptionT<Result<Option<i8>, String>> {
        if x.wrapping_add(f_seed) % 2 == 0 {
            OptionT::some(x.wrapping_add(1))
        } else {
            OptionT::none()
        }
    };

    let g = move |x: i8| -> OptionT<Result<Option<i8>, String>> {
        if x.wrapping_add(g_seed) % 3 == 0 {
            OptionT::some(x.wrapping_mul(2))
        } else {
            OptionT::none()
        }
    };

    let lhs = m.clone().flat_map(f).flat_map(g);
    let rhs = m.flat_map(|x| f(x).flat_map(g));

    lhs.run() == rhs.run()
}

// ============================================================================
// EitherT Laws
// ============================================================================

// EitherT<Option<Result<A, E>>> (Option as base monad)
// Note: EitherT wraps M<Result<A, E>>.
// Let's use Option as base monad: Option<Result<i8, String>>

fn eithert_left_identity(a: i8, f_seed: i8) -> bool {
    let f = move |x: i8| -> EitherT<Option<Result<i8, String>>> {
        if x.wrapping_add(f_seed) % 2 == 0 {
            EitherT::new(Some(Ok(x)))
        } else {
            EitherT::new(Some(Err("fail".to_string())))
        }
    };

    let lhs = EitherT::<Option<Result<i8, String>>>::new(Some(Ok(a))).flat_map(f);
    let rhs = f(a);

    lhs.run() == rhs.run()
}

fn eithert_right_identity(val: Option<Result<i8, String>>) -> bool {
    let m = EitherT::new(val.clone());
    let lhs = m.clone().flat_map(|x| EitherT::new(Some(Ok(x))));
    let rhs = m;

    lhs.run() == rhs.run()
}

fn eithert_associativity(val: Option<Result<i8, String>>, f_seed: i8, g_seed: i8) -> bool {
    let m = EitherT::<Option<Result<i8, String>>>::new(val);

    let f = move |x: i8| -> EitherT<Option<Result<i8, String>>> {
        if x.wrapping_add(f_seed) % 2 == 0 {
            EitherT::new(Some(Ok(x.wrapping_add(1))))
        } else {
            EitherT::new(Some(Err("f fail".to_string())))
        }
    };

    let g = move |x: i8| -> EitherT<Option<Result<i8, String>>> {
        if x.wrapping_add(g_seed) % 3 == 0 {
            EitherT::new(Some(Ok(x.wrapping_mul(2))))
        } else {
            EitherT::new(Some(Err("g fail".to_string())))
        }
    };

    let lhs = m.clone().flat_map(f).flat_map(g);
    let rhs = m.flat_map(|x| f(x).flat_map(g));

    lhs.run() == rhs.run()
}

// ============================================================================
// ReaderT Laws (Functional Equivalence)
// ============================================================================

// ReaderT<R, M> wraps R -> M<A>
// We verify laws by checking equality of results for random inputs.
// Base monad: Option

fn readert_left_identity(a: i8, r: i8, f_seed: i8) -> bool {
    let f = move |x: i8| -> ReaderT<i8, Option<i8>> {
        ReaderT::new(move |env: &i8| {
            if x.wrapping_add(*env).wrapping_add(f_seed) % 2 == 0 {
                Some(x)
            } else {
                None
            }
        })
    };

    let lhs = ReaderT::<i8, Option<i8>>::pure(a).flat_map(f);
    let rhs = f(a);

    lhs.run(&r) == rhs.run(&r)
}

fn readert_right_identity(a: i8, r: i8) -> bool {
    // Generate an arbitrary ReaderT (simulated)
    let make_m = |val: i8| ReaderT::new(move |env: &i8| Some(val.wrapping_add(*env)));

    let m = make_m(a);
    let lhs = m.flat_map(ReaderT::pure);
    let rhs = make_m(a); // Recreate because ReaderT is not Clone

    lhs.run(&r) == rhs.run(&r)
}

fn readert_associativity(a: i8, r: i8, f_seed: i8, g_seed: i8) -> bool {
    let make_m = |val: i8| ReaderT::new(move |env: &i8| Some(val.wrapping_add(*env)));

    let f = move |x: i8| -> ReaderT<i8, Option<i8>> {
        ReaderT::new(move |env: &i8| Some(x.wrapping_add(*env).wrapping_add(f_seed)))
    };

    let g = move |x: i8| -> ReaderT<i8, Option<i8>> {
        ReaderT::new(move |env: &i8| Some(x.wrapping_mul(*env).wrapping_add(g_seed)))
    };

    // lhs = (m >>= f) >>= g
    let m = make_m(a);
    let lhs = m.flat_map(f).flat_map(g);

    // rhs = m >>= (\x -> f x >>= g)
    let m = make_m(a);
    let rhs = m.flat_map(move |x| f(x).flat_map(g));

    lhs.run(&r) == rhs.run(&r)
}

// ============================================================================
// StateT Laws (Functional Equivalence)
// ============================================================================

// StateT<S, M> wraps S -> M<(A, S)>
// Base monad: Option

fn statet_left_identity(a: i8, s: i8, f_seed: i8) -> bool {
    let f = move |x: i8| -> StateT<i8, Option<(i8, i8)>> {
        StateT::new(move |state: i8| {
            if x.wrapping_add(state).wrapping_add(f_seed) % 2 == 0 {
                Some((x, state.wrapping_add(1)))
            } else {
                None
            }
        })
    };

    let lhs = StateT::<i8, Option<(i8, i8)>>::pure(a).flat_map(f);
    let rhs = f(a);

    lhs.run(s) == rhs.run(s)
}

fn statet_right_identity(a: i8, s: i8) -> bool {
    let make_m = |val: i8| {
        StateT::new(move |state: i8| Some((val.wrapping_add(state), state.wrapping_add(1))))
    };

    let m = make_m(a);
    let lhs = m.flat_map(StateT::pure);
    let rhs = make_m(a);

    lhs.run(s) == rhs.run(s)
}

fn statet_associativity(a: i8, s: i8, f_seed: i8, g_seed: i8) -> bool {
    let make_m = |val: i8| {
        StateT::new(move |state: i8| Some((val.wrapping_add(state), state.wrapping_add(1))))
    };

    let f = move |x: i8| -> StateT<i8, Option<(i8, i8)>> {
        StateT::new(move |state: i8| Some((x.wrapping_add(1), state.wrapping_add(f_seed))))
    };

    let g = move |x: i8| -> StateT<i8, Option<(i8, i8)>> {
        StateT::new(move |state: i8| Some((x.wrapping_mul(2), state.wrapping_add(g_seed))))
    };

    let m = make_m(a);
    let lhs = m.flat_map(f).flat_map(g);

    let m = make_m(a);
    let rhs = m.flat_map(move |x| f(x).flat_map(g));

    lhs.run(s) == rhs.run(s)
}

// ============================================================================
// Test Runners
// ============================================================================

#[test]
fn test_optiont_laws() {
    quickcheck(optiont_left_identity as fn(i8, i8) -> bool);
    quickcheck(optiont_right_identity as fn(Option<i8>) -> bool);
    quickcheck(optiont_associativity as fn(Option<i8>, i8, i8) -> bool);
}

#[test]
fn test_eithert_laws() {
    quickcheck(eithert_left_identity as fn(i8, i8) -> bool);
    quickcheck(eithert_right_identity as fn(Option<Result<i8, String>>) -> bool);
    quickcheck(eithert_associativity as fn(Option<Result<i8, String>>, i8, i8) -> bool);
}

#[test]
fn test_readert_laws() {
    quickcheck(readert_left_identity as fn(i8, i8, i8) -> bool);
    quickcheck(readert_right_identity as fn(i8, i8) -> bool);
    quickcheck(readert_associativity as fn(i8, i8, i8, i8) -> bool);
}

#[test]
fn test_statet_laws() {
    quickcheck(statet_left_identity as fn(i8, i8, i8) -> bool);
    quickcheck(statet_right_identity as fn(i8, i8) -> bool);
    quickcheck(statet_associativity as fn(i8, i8, i8, i8) -> bool);
}
