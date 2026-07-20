//! Macros all collected into a single module so that the order of `mod`
//! statements in `lib.rs` does not matter.
//!
//! This module includes:
//! - `HList` construction and pattern matching macros
//! - Disiunctio type macros
//! - Polymorphic function macros
//! - Monadic do-notation (`mdo!`)
//! - Function combinators (`compose!`, `pipe!`, `curry!`)

/// Returns an `HList` based on the values passed in.
///
/// Helps to avoid having to write nested `Coniunctio`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::hlist;
/// # fn main() {
/// let h = hlist![13.5f32, "hello", Some(41)];
/// let (h1, (h2, h3)) = h.into_tuple2();
/// assert_eq!(h1, 13.5f32);
/// assert_eq!(h2, "hello");
/// assert_eq!(h3, Some(41));
///
/// // Also works when you have trailing commas
/// let h4 = hlist!["yo",];
/// let h5 = hlist![13.5f32, "hello", Some(41),];
/// assert_eq!(h4, hlist!["yo"]);
/// assert_eq!(h5, hlist![13.5f32, "hello", Some(41)]);
///
/// // Use "...tail" to append an existing list at the end
/// let h6 = hlist![12, ...h5];
/// assert_eq!(h6, hlist![12, 13.5f32, "hello", Some(41)]);
/// # }
/// ```
#[macro_export]
macro_rules! hlist {
    () => { $crate::hlist::Nihil };
    (...$rest:expr) => { $rest };
    ($a:expr) => { $crate::hlist![$a,] };
    ($a:expr, $($tok:tt)*) => {
        $crate::hlist::Coniunctio {
            head: $a,
            tail: $crate::hlist![$($tok)*],
        }
    };
}

/// Macro for pattern-matching on `HList`s.
///
/// Taken from <https://github.com/tbu-/rust-rfcs/blob/master/text/0873-type-macros.md>
///
/// # Examples
///
/// ```rust
/// use ordofp_core::{hlist, coniunctio_pat};
/// # fn main() {
/// let h = hlist![13.5f32, "hello", Some(41)];
/// let coniunctio_pat![a1, a2, a3] = h;
/// assert_eq!(a1, 13.5f32);
/// assert_eq!(a2, "hello");
/// assert_eq!(a3, Some(41));
///
/// // Use "...tail" to match the rest of the list
/// let coniunctio_pat![b_head, ...b_tail] = h;
/// assert_eq!(b_head, 13.5f32);
/// assert_eq!(b_tail, hlist!["hello", Some(41)]);
///
/// // You can also use "..." to just ignore the rest.
/// let coniunctio_pat![c, ...] = h;
/// assert_eq!(c, 13.5f32);
/// # }
/// ```
#[macro_export]
macro_rules! coniunctio_pat {
    () => { $crate::hlist::Nihil };
    (...) => { _ };
    (...$rest:pat) => { $rest };
    (_) => { $crate::coniunctio_pat![_,] };
    ($a:pat) => { $crate::coniunctio_pat![$a,] };
    (_, $($tok:tt)*) => {
        $crate::hlist::Coniunctio {
            tail: $crate::coniunctio_pat![$($tok)*],
            ..
        }
    };
    ($a:pat, $($tok:tt)*) => {
        $crate::hlist::Coniunctio {
            head: $a,
            tail: $crate::coniunctio_pat![$($tok)*],
        }
    };
}

/// Returns a type signature for an `HList` of the provided types
///
/// This is a type macro (introduced in Rust 1.13) that makes it easier
/// to write nested type signatures.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::{hlist, HList};
/// # fn main() {
/// let h: HList!(f32, &str, Option<i32>) = hlist![13.5f32, "hello", Some(41)];
///
/// // Use "...Tail" to append another HList type at the end.
/// let h: HList!(f32, ...HList!(&str, Option<i32>)) = hlist![13.5f32, "hello", Some(41)];
/// # }
/// ```
#[macro_export]
macro_rules! HList {
    () => { $crate::hlist::Nihil };
    (...$Rest:ty) => { $Rest };
    ($A:ty) => { $crate::HList![$A,] };
    ($A:ty, $($tok:tt)*) => {
        $crate::hlist::Coniunctio<$A, $crate::HList![$($tok)*]>
    };
}

/// Returns a type signature for a Disiunctio of the provided types.
///
/// This is a type macro (introduced in Rust 1.13) that makes it easier
/// to write nested type signatures.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// use ordofp_core::Disiunctio;
///
/// type I32Bool = Disiunctio!(i32, bool);
/// let co1 = I32Bool::inject(3);
///
/// // Use ...Tail to append another Disiunctio at the end.
/// let co2 = <Disiunctio!(&str, String, ...I32Bool)>::inject(3);
/// # }
/// ```
#[macro_export]
macro_rules! Disiunctio {
    () => { $crate::disiunctio::Absurdum };
    (...$Rest:ty) => { $Rest };
    ($A:ty) => { $crate::Disiunctio![$A,] };
    ($A:ty, $($tok:tt)*) => {
        $crate::disiunctio::Disiunctio<$A, $crate::Disiunctio![$($tok)*]>
    };
}

/// Used for creating a Field
///
/// There are 3 forms of this macro:
///
/// * Create an instance of the `Field` struct with a tuple name type
///   and any given value. The runtime-retrievable static name
///   field will be set to the the concatenation of the types passed in the
///   tuple type used as the first argument.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::field;
/// # fn main() {
/// // The static name is the concatenation of the type-level char names,
/// // so it reflects their (multi-character) identifiers here.
/// let labelled = field![(Ln, La, Lm, Le), "joe"];
/// assert_eq!(labelled.name, "LnLaLmLe");
/// assert_eq!(labelled.value, "joe")
/// # }
/// ```
///
/// * Create an instance of the `Field` struct with a custom, non-tuple
///   name type and a value. The runtime-retrievable static name field
///   will be set to the stringified version of the type provided.
///
/// ```rust
/// # fn main() {
/// use ordofp_core::field;
/// enum first_name {}
/// let labelled = field![first_name, "Joe"];
/// assert_eq!(labelled.name, "first_name");
/// assert_eq!(labelled.value, "Joe");
/// # }
/// ```
///
/// * Create an instance of the `Field` struct with any name type and value,
///   _and_ a custom name, passed as the last argument in the macro
///
/// ```rust
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::field;
/// # fn main() {
/// // useful aliasing of our type-level string
/// type age = (La, Lg, Le);
/// let labelled = field![age, 30, "Age"];
/// assert_eq!(labelled.name, "Age");
/// assert_eq!(labelled.value, 30);
/// # }
/// ```
#[macro_export]
macro_rules! field {
    // No name provided and type is a tuple
    (($($repeated: ty),*), $value: expr) => {
        $crate::field!( ($($repeated),*), $value, concat!( $(stringify!($repeated)),* ) )
    };
    // No name provided and type is a tuple, but with trailing commas
    (($($repeated: ty,)*), $value: expr) => {
        $crate::field!( ($($repeated),*), $value )
    };
    // We are provided any type, with no stable name
    ($name_type: ty, $value: expr) => {
        $crate::field!( $name_type, $value, stringify!($name_type) )
    };
    // We are provided any type, with a stable name
    ($name_type: ty, $value: expr, $name: expr) => {
        $crate::labelled::field_with_name::<$name_type,_>($name, $value)
    }
}

/// Returns a polymorphic function for use with mapping/folding heterogeneous
/// types.
///
/// This macro is intended for use with simple scenarios, and doesn't handle
/// trait implementation bounds or where clauses (it might in the future when
/// procedural macros land). If it doesn't work for you, simply implement
/// Func on your own.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// use ordofp_core::{Disiunctio, functio_poly};
/// type I32F32Str<'a> = Disiunctio!(i32, f32, &'a str);
///
/// let co1 = I32F32Str::inject("lollerskates");
/// let folded = co1.fold(functio_poly!(
///   ['a] |_x: &'a str| -> i8 { 1 },
///   |_x: i32| -> i8 { 2 },
///   |_f: f32| -> i8 { 3 },
/// ));
///
/// assert_eq!(folded, 1);
/// # }
/// ```
#[macro_export]
macro_rules! functio_poly {
    // encountered first func w/ type params
    ([$($tparams: tt),*] |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block , $($rest: tt)*)
    => { $crate::functio_poly!(
       p~ [$($tparams, )*] |$arg: $arg_typ| -> $ret_typ $body, ~p  f~ ~f $($rest)*
    )};
    // encountered first func w/ type params, trailing comma on tparams
    ([$($tparams: tt, )*] |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block , $($rest: tt)*)
    => { $crate::functio_poly!(
       p~ [$($tparams, )*] |$arg: $arg_typ| -> $ret_typ $body, ~p  f~ ~f $($rest)*
    )};
    // encountered first func w/o type params
    (|$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block, $($rest: tt)*)
    => { $crate::functio_poly!(
       p~ ~p  f~ |$arg: $arg_typ| -> $ret_typ $body, ~f $($rest)*
    )};

    // encountered non-first func w/ type params
    (p~ $([$($pars: tt, )*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty $p_body: block , )* ~p f~ $(|$f_args: ident : $f_arg_typ: ty| -> $f_ret_typ: ty $f_body: block , )* ~f [$($tparams: tt),*] |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block , $($rest: tt)*)
    => { $crate::functio_poly!(
       p~ [$($tparams, )*] |$arg: $arg_typ| -> $ret_typ $body, $( [$($pars, )*] |$p_args: $p_arg_typ| -> $p_ret_typ $p_body, )* ~p  f~ $(|$f_args: $f_arg_typ| -> $f_ret_typ $f_body, )* ~f $($rest)*
    )};
    // encountered non-first func w/ type params, trailing comma in tparams
    (p~ $([$($pars: tt, )*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty { $p_body: block }, )* ~p f~ $(|$f_args: ident : $f_arg_typ: ty| -> $f_ret_typ: ty $f_body: block, )* ~f [$($tparams: tt, )*] |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block, $($rest: tt)*)
    => { $crate::functio_poly!(
       p~ [$($tparams, )*] |$arg: $arg_typ| -> $ret_typ $body, $( [$($pars, )*] |$p_args: $p_arg_typ| -> $p_ret_typ $p_body, )* ~p  f~ $(|$f_args: $f_arg_typ| -> $f_ret_typ $f_body, )* ~f $($rest)*
    )};
    // encountered non-first func w/o type params
    (p~ $([$($pars: tt, )*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty $p_body: block, )* ~p f~ $(|$f_args: ident : $f_arg_typ: ty| -> $f_ret_typ: ty $f_body: block, )* ~f |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block, $($rest: tt)*)
    => { $crate::functio_poly!(
       p~ $( [$($pars, )*] |$p_args: $p_arg_typ| -> $p_ret_typ $p_body, )* ~p  f~ |$arg: $arg_typ| -> $ret_typ $body, $(|$f_args: $f_arg_typ| -> $f_ret_typ $f_body, )* ~f $($rest)*
    )};

    // last w/ type params, for when there is no trailing comma on the funcs...
    (p~ $([$($pars: tt, )*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty $p_body: block, )* ~p f~ $(|$f_args: ident : $f_arg_typ: ty| -> $f_ret_typ: ty $f_body: block, )* ~f [$($tparams: tt),*] |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block)
    => { $crate::functio_poly!(
       p~ [$($tparams, )*] |$arg: $arg_typ| -> $ret_typ $body, $( [$($pars, )*] |$p_args: $p_arg_typ| -> $p_ret_typ $p_body, )* ~p  f~ $(|$f_args: $f_arg_typ| -> $f_ret_typ $f_body, )* ~f
    )};
    // last w/ type params, for when there is a trailing comma in tparams, but no trailing comma on the funcs..
    (p~ $([$($pars: tt, )*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty $p_body: block, )* ~p f~ $(|$f_args: ident : $f_arg_typ: ty| -> $f_ret_typ: ty $f_body: block, )* ~f [$($tparams: tt, )*] |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block)
    => { $crate::functio_poly!(
       p~ [$($tparams, )*] |$arg: $arg_typ| -> $ret_typ $body, $( [$($pars, )*] |$p_args: $p_arg_typ| -> $p_ret_typ $p_body, )* ~p  f~ $(|$f_args: $f_arg_typ| -> $f_ret_typ $f_body, )* ~f
    )};
    // last w/o type params, for when there is no trailing comma on the funcs...
    (p~ $([$($pars: tt)*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty $p_body: block, )* ~p f~ $(|$f_args: ident : $f_arg_typ: ty| -> $f_ret_typ: ty $f_body: block, )* ~f |$arg: ident : $arg_typ: ty| -> $ret_typ: ty $body: block)
    => { $crate::functio_poly!(
       p~ $( [$($pars, )*] |$p_args: $p_arg_typ| -> $p_ret_typ $p_body, )* ~p  f~ |$arg: $arg_typ| -> $ret_typ $body, $(|$f_args: $f_arg_typ| -> $f_ret_typ $f_body, )* ~f
    )};

    // unroll
    (p~ $([$($pars: tt, )*] |$p_args: ident : $p_arg_typ: ty| -> $p_ret_typ: ty $p_body: block, )* ~p f~ $(|$args: ident : $arg_typ: ty| -> $ret_typ: ty $body: block, )* ~f) => {{
        struct F;
        $(
            impl<$($pars,)*> $crate::traits::Func<$p_arg_typ> for F {
                type Output = $p_ret_typ;

                #[inline(always)]
                fn call($p_args: $p_arg_typ) -> Self::Output { $p_body }
            }
        )*
        $(
            impl $crate::traits::Func<$arg_typ> for F {
                type Output = $ret_typ;

                #[inline(always)]
                fn call($args: $arg_typ) -> Self::Output { $body }
            }
        )*
        $crate::traits::Poly(F)
    }}
}

// =============================================================================
// MONADIC DO-NOTATION
// =============================================================================

/// Monadic do-notation for composing monadic operations.
///
/// This macro provides Haskell-like do-notation for working with monads such as
/// `Option`, `Result`, or any type that implements `and_then`.
///
/// # Syntax
///
/// ```text
/// mdo! {
///     let x = bind expression;   // Bind: extract value from monad
///     let y = pure expression;   // Let: regular variable binding
///     expression                 // Final expression: returns the monad
/// }
/// ```
///
/// # Examples
///
/// ## With Option
///
/// ```rust
/// use ordofp_core::mdo;
///
/// fn safe_div(a: i32, b: i32) -> Option<i32> {
///     if b == 0 { None } else { Some(a / b) }
/// }
///
/// let result = mdo! {
///     let x = bind Some(10);
///     let y = bind Some(2);
///     let z = bind safe_div(x, y);
///     Some(z * 3)
/// };
///
/// assert_eq!(result, Some(15));
/// ```
///
/// ## With Result
///
/// ```rust
/// use ordofp_core::mdo;
///
/// fn parse_int(s: &str) -> Result<i32, &'static str> {
///     s.parse().map_err(|_| "parse error")
/// }
///
/// let result: Result<i32, &str> = mdo! {
///     let x = bind parse_int("10");
///     let y = bind parse_int("5");
///     let sum = pure x + y;
///     Ok(sum)
/// };
///
/// assert_eq!(result, Ok(15));
/// ```
///
/// ## Short-circuit on failure
///
/// ```rust
/// use ordofp_core::mdo;
///
/// let result = mdo! {
///     let x = bind Some(10);
///     let y = bind None::<i32>;  // This will short-circuit
///     Some(x + y)
/// };
///
/// assert_eq!(result, None);
/// ```
///
/// ## Using pure bindings
///
/// ```rust
/// use ordofp_core::mdo;
///
/// let result = mdo! {
///     let x = bind Some(10);
///     let doubled = pure x * 2;
///     let y = bind Some(5);
///     Some(doubled + y)
/// };
///
/// assert_eq!(result, Some(25));
/// ```
#[macro_export]
macro_rules! mdo {
    // Final expression - wrap and return
    ($e:expr) => { $e };

    // Monadic bind: let x = bind expr;
    (let $x:ident = bind $e:expr; $($rest:tt)+) => {
        $e.and_then(move |$x| $crate::mdo!($($rest)+))
    };

    // Pure binding: let x = pure expr;
    (let $x:ident = pure $e:expr; $($rest:tt)+) => {{
        let $x = $e;
        $crate::mdo!($($rest)+)
    }};

    // Regular let binding (backwards compatibility)
    (let $p:pat = $e:expr; $($rest:tt)+) => {{
        let $p = $e;
        $crate::mdo!($($rest)+)
    }};

    // Statement (for side effects)
    ($s:stmt; $($rest:tt)+) => {{
        $s;
        $crate::mdo!($($rest)+)
    }};
}

// =============================================================================
// FUNCTION COMBINATORS
// =============================================================================

/// Compose functions from right to left.
///
/// (f ∘ g ∘ h)(x) = f(g(h(x)))
///
/// # Example
///
/// ```rust
/// use ordofp_core::compose;
///
/// let f = |x: i32| x + 1;
/// let g = |x: i32| x * 2;
/// let h = |x: i32| x - 3;
///
/// let composed = compose!(f, g, h);
/// // composed(10) = f(g(h(10))) = f(g(7)) = f(14) = 15
/// assert_eq!(composed(10), 15);
/// ```
#[macro_export]
macro_rules! compose {
    ($f:expr) => { $f };
    ($f:expr, $($rest:expr),+) => {
        |x| $f($crate::compose!($($rest),+)(x))
    };
}

/// Pipe a value through a series of functions from left to right.
///
/// x |> f |> g |> h = h(g(f(x)))
///
/// # Example
///
/// ```rust
/// use ordofp_core::pipe;
///
/// let f = |x: i32| x + 1;
/// let g = |x: i32| x * 2;
/// let h = |x: i32| x - 3;
///
/// let result = pipe!(10, f, g, h);
/// // result = h(g(f(10))) = h(g(11)) = h(22) = 19
/// assert_eq!(result, 19);
/// ```
#[macro_export]
macro_rules! pipe {
    ($x:expr) => { $x };
    ($x:expr, $f:expr) => { $f($x) };
    ($x:expr, $f:expr, $($rest:expr),+) => {
        $crate::pipe!($f($x), $($rest),+)
    };
}

/// Curry a function of 2 arguments.
///
/// Converts `fn(A, B) -> C` to `fn(A) -> fn(B) -> C`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::curry2;
///
/// let add = |a: i32, b: i32| a + b;
/// let curried = curry2!(add);
///
/// let add5 = curried(5);
/// assert_eq!(add5(10), 15);
/// ```
#[macro_export]
macro_rules! curry2 {
    ($f:expr) => {
        move |x| move |y| $f(x, y)
    };
}

/// Curry a function of 3 arguments.
///
/// Converts `fn(A, B, C) -> D` to `fn(A) -> fn(B) -> fn(C) -> D`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::curry3;
///
/// let add3 = |a: i32, b: i32, c: i32| a + b + c;
/// let curried = curry3!(add3);
///
/// let partial = curried(1)(2);
/// assert_eq!(partial(3), 6);
/// ```
#[macro_export]
macro_rules! curry3 {
    ($f:expr) => {
        move |x| move |y| move |z| $f(x, y, z)
    };
}

/// Flip the arguments of a binary function.
///
/// flip(f)(x, y) = f(y, x)
///
/// # Example
///
/// ```rust
/// use ordofp_core::flip;
///
/// let sub = |a: i32, b: i32| a - b;
/// let flipped = flip!(sub);
///
/// assert_eq!(sub(10, 3), 7);
/// assert_eq!(flipped(10, 3), -7);  // 3 - 10
/// ```
#[macro_export]
macro_rules! flip {
    ($f:expr) => {
        |y, x| $f(x, y)
    };
}

/// Create a constant function that ignores its argument.
///
/// constant(x) = |_| x
///
/// # Example
///
/// ```rust
/// use ordofp_core::constant;
///
/// let ignore_int = constant!(42);
/// assert_eq!(ignore_int(100), 42);
///
/// // A single closure value is monomorphic, so a fresh instance is needed
/// // when the ignored argument's type differs — but it works identically.
/// let ignore_str = constant!(42);
/// assert_eq!(ignore_str("hello"), 42);
/// ```
#[macro_export]
macro_rules! constant {
    ($x:expr) => {
        |_| $x
    };
}

// =============================================================================
// COLD PATH MACROS
// =============================================================================

/// A cold-path panic that ensures the panic logic is out-of-line.
///
/// This helps keep the hot path compact and improves instruction cache efficiency.
#[macro_export]
macro_rules! cold_panic {
    ($($arg:tt)*) => {{
        #[cold]
        #[inline(never)]
        fn cold_panic_impl(args: core::fmt::Arguments) -> ! {
            panic!("{}", args)
        }
        cold_panic_impl(format_args!($($arg)*))
    }};
}

/// A panic that is marked as unlikely and cold.
#[macro_export]
macro_rules! unlikely_panic {
    ($cond:expr, $($arg:tt)*) => {
        if $cond {
            $crate::cold_panic!($($arg)*);
        }
    };
}

// =============================================================================
// ASYNC MONADIC DO-NOTATION AND COMBINATORS
// =============================================================================

/// Async monadic do-notation for composing async monadic operations.
///
/// *"Fac quod debes"* - Do what you must
///
/// This macro provides Haskell-like do-notation for working with async monads such as
/// `Futurus`, `LectorAsync`, `StatusAsync`, or any async type that implements appropriate
/// async bind operations.
///
/// # Syntax
///
/// ```text
/// mdo_async! {
///     let x = bind expression;    // Bind: async monad bind (uses and_then or flat_map)
///     let y = await expression;   // Await: await a future
///     let z = pure expression;    // Pure: regular variable binding
///     expression                  // Final expression: returns the result
/// }
/// ```
///
/// # Bind Semantics
///
/// The `bind` keyword uses `.and_then()` for types that implement it (like `Option`, `Result`)
/// or `.flat_map()` for custom async monads. The continuation is wrapped in `async move` for
/// async compatibility.
///
/// # Examples
///
/// ## With Option (sync bind, async context)
///
/// ```rust
/// use ordofp_core::mdo_async;
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// async fn example() -> Option<i32> {
///     mdo_async! {
///         let x = bind Some(10);
///         let y = bind Some(5);
///         Some(x + y)
///     }
/// }
///
/// assert_eq!(drive(example()), Some(15));
/// ```
///
/// ## With Futures and await
///
/// ```rust
/// use ordofp_core::mdo_async;
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// async fn fetch_data(id: i32) -> String {
///     format!("data-{}", id)
/// }
///
/// async fn example() -> String {
///     mdo_async! {
///         let id = pure 42;
///         let data = await fetch_data(id);
///         let processed = pure data.to_uppercase();
///         processed
///     }
/// }
///
/// assert_eq!(drive(example()), "DATA-42");
/// ```
///
/// ## With `LectorAsync` (async reader monad)
///
/// ```rust
/// use ordofp_core::mdo_async;
/// use ordofp_core::transformers::async_transforms::{LectorAsync, MonadTransformerAsync};
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// #[derive(Clone)]
/// struct Config {
///     user_id: i32,
/// }
///
/// async fn fetch_user(user_id: i32) -> String {
///     format!("user-{}", user_id)
/// }
///
/// // Note: `bind`'s continuation is a plain (non-async) closure, so once a `bind`
/// // step appears, later steps can no longer `await` directly — lift the async
/// // work into the monad first (e.g. via `lift_async`) and `bind` on that instead.
/// let computation = mdo_async! {
///     let config = bind LectorAsync::<Config, Config>::ask();
///     let data = bind <LectorAsync<Config, String> as MonadTransformerAsync>::lift_async(
///         fetch_user(config.user_id)
///     );
///     LectorAsync::purus(data)
/// };
///
/// let result = drive(computation.run(Config { user_id: 42 }));
/// assert_eq!(result, "user-42");
/// ```
///
/// ## Mixing bind, await, and pure
///
/// ```rust
/// use ordofp_core::mdo_async;
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// // Note: `await`/`pure` steps must come before the first `bind` step — once a
/// // `bind` is used, its continuation is a plain closure and can no longer `await`.
/// async fn process() -> Option<String> {
///     mdo_async! {
///         let doubled = pure 10 * 2;
///         let data = await async { format!("value: {}", doubled) };
///         let x = bind Some(data);
///         let y = bind Some(x.len());
///         Some(format!("{x}/{y}"))
///     }
/// }
///
/// assert_eq!(drive(process()), Some("value: 20/9".to_string()));
/// ```
#[macro_export]
#[cfg(feature = "async")]
macro_rules! mdo_async {
    // Final expression - wrap and return
    ($e:expr) => { $e };

    // Monadic bind using and_then (for Option, Result, etc.)
    // let x = bind expr;
    (let $x:ident = bind $e:expr; $($rest:tt)+) => {
        $e.and_then(move |$x| $crate::mdo_async!($($rest)+))
    };

    // Await pattern: let x = await expr;
    // Awaits a future and continues with the result
    (let $x:ident = await $e:expr; $($rest:tt)+) => {{
        let $x = $e.await;
        $crate::mdo_async!($($rest)+)
    }};

    // Pure binding: let x = pure expr;
    // Regular let binding (no monadic extraction)
    (let $x:ident = pure $e:expr; $($rest:tt)+) => {{
        let $x = $e;
        $crate::mdo_async!($($rest)+)
    }};

    // Regular let binding (backwards compatibility)
    (let $p:pat = $e:expr; $($rest:tt)+) => {{
        let $p = $e;
        $crate::mdo_async!($($rest)+)
    }};

    // Statement (for side effects)
    ($s:stmt; $($rest:tt)+) => {{
        $s;
        $crate::mdo_async!($($rest)+)
    }};
}

/// Compose async functions from right to left.
///
/// (f ∘ g ∘ h)(x) = f(g(h(x)).await).await
///
/// Each function must return a Future. The composition awaits each result
/// before passing it to the next function.
///
/// # Example
///
/// ```rust
/// use ordofp_core::compose_async;
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// async fn add_one(x: i32) -> i32 { x + 1 }
/// async fn double(x: i32) -> i32 { x * 2 }
/// async fn subtract_three(x: i32) -> i32 { x - 3 }
///
/// let composed = compose_async!(add_one, double, subtract_three);
/// // composed(10) = add_one(double(subtract_three(10).await).await).await
/// //              = add_one(double(7).await).await
/// //              = add_one(14).await
/// //              = 15
/// let result = drive(composed(10));
/// assert_eq!(result, 15);
/// ```
#[macro_export]
#[cfg(feature = "async")]
macro_rules! compose_async {
    ($f:expr) => {
        |x| $f(x)
    };
    ($f:expr, $($rest:expr),+) => {
        |x| async move {
            let intermediate = $crate::compose_async!($($rest),+)(x).await;
            $f(intermediate).await
        }
    };
}

/// Pipe a value through a series of async functions from left to right.
///
/// x |> f |> g |> h = h(g(f(x).await).await).await
///
/// Each function must return a Future. The pipe awaits each result
/// before passing it to the next function.
///
/// # Example
///
/// ```rust
/// use ordofp_core::pipe_async;
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// async fn add_one(x: i32) -> i32 { x + 1 }
/// async fn double(x: i32) -> i32 { x * 2 }
/// async fn subtract_three(x: i32) -> i32 { x - 3 }
///
/// let result = drive(pipe_async!(10, add_one, double, subtract_three));
/// // result = subtract_three(double(add_one(10).await).await).await
/// //        = subtract_three(double(11).await).await
/// //        = subtract_three(22).await
/// //        = 19
/// assert_eq!(result, 19);
/// ```
#[macro_export]
#[cfg(feature = "async")]
macro_rules! pipe_async {
    ($x:expr) => { async move { $x } };
    ($x:expr, $f:expr) => { $f($x) };
    ($x:expr, $f:expr, $($rest:expr),+) => {
        async move {
            let intermediate = $f($x).await;
            $crate::pipe_async!(intermediate, $($rest),+).await
        }
    };
}

/// Chain async functions from left to right, returning a composed function.
///
/// Similar to `compose_async` but with left-to-right order (like Haskell's >>>).
///
/// # Example
///
/// ```rust
/// use ordofp_core::chain_async;
///
/// # fn drive<F: core::future::Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = core::task::Context::from_waker(core::task::Waker::noop());
/// #     loop {
/// #         if let core::task::Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
///
/// async fn parse(s: &str) -> i32 { s.parse().unwrap() }
/// async fn double(x: i32) -> i32 { x * 2 }
/// async fn to_string(x: i32) -> String { x.to_string() }
///
/// let process = chain_async!(parse, double, to_string);
/// let result = drive(process("21"));
/// assert_eq!(result, "42");
/// ```
#[macro_export]
#[cfg(feature = "async")]
macro_rules! chain_async {
    ($f:expr) => {
        |x| $f(x)
    };
    ($f:expr, $($rest:expr),+) => {
        |x| async move {
            let intermediate = $f(x).await;
            $crate::chain_async!($($rest),+)(intermediate).await
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    // The never-executed `let _: Disiunctio![…] = panic!()` blocks are
    // typecheck proofs; the diverging expression is the cheapest inhabitant.
    #[allow(clippy::diverging_sub_expression)]
    fn trailing_commas() {
        use crate::test_structs::unit_copy::{A, B};

        let coniunctio_pat![]: HList![] = hlist![];
        let coniunctio_pat![A]: HList![A] = hlist![A];
        let coniunctio_pat![A,]: HList![A,] = hlist![A,];
        let coniunctio_pat![A, B]: HList![A, B] = hlist![A, B];
        let coniunctio_pat![A, B,]: HList![A, B,] = hlist![A, B,];

        let falsum = || false;
        if falsum() {
            let _: Disiunctio![] = panic!();
        }
        if falsum() {
            let _: Disiunctio![A] = panic!();
        }
        if falsum() {
            let _: Disiunctio![A,] = panic!();
        }
        if falsum() {
            let _: Disiunctio![A, B] = panic!();
        }
        if falsum() {
            let _: Disiunctio![A, B,] = panic!();
        }
    }

    #[test]
    fn ellipsis_tail() {
        use crate::disiunctio::Disiunctio;
        use crate::test_structs::unit_copy::{A, B, C};

        // hlist: accepted locations, and consistency between macros
        let coniunctio_pat![...coniunctio_pat![C]]: HList![...HList![C]] = { hlist![...hlist![C]] };
        let coniunctio_pat![A, ...coniunctio_pat![C]]: HList![A, ...HList![C]] =
            { hlist![A, ...hlist![C]] };
        let coniunctio_pat![A, B, ...coniunctio_pat![C]]: HList![A, B, ...HList![C]] =
            { hlist![A, B, ...hlist![C]] };

        // hlist: ellipsis semantics
        //   (by pairing an ellipsis call with a non-ellipsis call)
        let coniunctio_pat![A, B, C] = hlist![A, ...hlist![B, C]];
        let coniunctio_pat![A, ...coniunctio_pat![B, C]] = hlist![A, B, C];

        // disiunctio: accepted locations and semantics
        let choice: Disiunctio![A, B, C] = Disiunctio::inject(A);
        let _: Disiunctio![...Disiunctio![A, B, C]] = choice;
        let _: Disiunctio![A, ...Disiunctio![B, C]] = choice;
        let _: Disiunctio![A, B, ...Disiunctio![C]] = choice;
    }

    #[test]
    fn ellipsis_ignore() {
        use crate::test_structs::unit_copy::{A, B, C, D, E};

        // '...' accepted locations
        let coniunctio_pat![...] = hlist![A, B, C, D, E];
        let coniunctio_pat![A, ...] = hlist![A, B, C, D, E];
        let coniunctio_pat![A, B, ...] = hlist![A, B, C, D, E];
    }

    #[test]
    fn functio_poly_macro_test() {
        let h = hlist![9000, "joe", 41f32, "schmoe", 50];
        let h2 = h.map(functio_poly!(
            |x: i32| -> bool { x > 100 },
            |_x: f32| -> &'static str { "dummy" },
            ['a] |x: &'a str| -> usize { x.len() }
        ));
        assert_eq!(h2, hlist![true, 3, "dummy", 6, false]);
    }

    #[test]
    fn functio_poly_macro_disiunctio_test() {
        type I32F32StrBool<'a> = Disiunctio!(i32, f32, &'a str);

        let co1 = I32F32StrBool::inject("lollerskates");
        let folded = co1.fold(functio_poly!(
            ['a] |_x: &'a str| -> i8 { 1 },
            |_x: i32| -> i8 { 2 },
            |_f: f32| -> i8 { 3 },
        ));
        assert_eq!(folded, 1);
    }

    #[test]
    fn functio_poly_macro_trailing_commas_test() {
        let h = hlist![9000, "joe", 41f32, "schmoe", 50];
        let h2 = h.map(functio_poly!(
            |x: i32| -> bool { x > 100 },
            |_x: f32| -> &'static str { "dummy" },
            ['a,] |x: &'a str| -> usize { x.len() },
        ));
        assert_eq!(h2, hlist![true, 3, "dummy", 6, false]);
    }

    #[test]
    fn functio_poly_macro_multiline_bodies_test() {
        let h = hlist![9000, 1, -1];
        let h2 = h.map(functio_poly!(|x: i32| -> bool {
            let a = if x > 100 { 1 } else { -1 };
            a > 0
        },));
        assert_eq!(h2, hlist![true, false, false]);
    }

    #[test]
    #[deny(clippy::unneeded_field_pattern)]
    fn unneeded_field_pattern() {
        let coniunctio_pat![_, _] = hlist![1, 2];
        let coniunctio_pat![foo, _, baz] = hlist!["foo", "bar", "baz"];
        assert_eq!(foo, "foo");
        assert_eq!(baz, "baz");
    }

    // ==================== mdo! macro tests ====================

    #[test]
    fn test_mdo_option_success() {
        let result = mdo! {
            let x = bind Some(10);
            let y = bind Some(5);
            Some(x + y)
        };
        assert_eq!(result, Some(15));
    }

    #[test]
    fn test_mdo_option_short_circuit() {
        let result = mdo! {
            let x = bind Some(10);
            let y = bind None::<i32>;
            Some(x + y)
        };
        assert_eq!(result, None);
    }

    #[test]
    fn test_mdo_option_with_pure() {
        let result = mdo! {
            let x = bind Some(10);
            let doubled = pure x * 2;
            let y = bind Some(5);
            Some(doubled + y)
        };
        assert_eq!(result, Some(25));
    }

    #[test]
    fn test_mdo_result_success() {
        fn parse(s: &str) -> Result<i32, &'static str> {
            s.parse().map_err(|_| "parse error")
        }

        let result: Result<i32, &str> = mdo! {
            let x = bind parse("10");
            let y = bind parse("5");
            Ok(x + y)
        };
        assert_eq!(result, Ok(15));
    }

    #[test]
    fn test_mdo_result_failure() {
        fn parse(s: &str) -> Result<i32, &'static str> {
            s.parse().map_err(|_| "parse error")
        }

        let result: Result<i32, &str> = mdo! {
            let x = bind parse("10");
            let y = bind parse("not_a_number");
            Ok(x + y)
        };
        assert_eq!(result, Err("parse error"));
    }

    // ==================== compose! macro tests ====================

    #[test]
    fn test_compose_single() {
        let f = |x: i32| x + 1;
        let composed = compose!(f);
        assert_eq!(composed(10), 11);
    }

    #[test]
    fn test_compose_multiple() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = |x: i32| x - 3;

        // compose!(f, g, h)(10) = f(g(h(10))) = f(g(7)) = f(14) = 15
        let composed = compose!(f, g, h);
        assert_eq!(composed(10), 15);
    }

    // ==================== pipe! macro tests ====================

    #[test]
    fn test_pipe_single() {
        let result = pipe!(10);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_pipe_one_function() {
        let f = |x: i32| x + 1;
        let result = pipe!(10, f);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_pipe_multiple() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = |x: i32| x - 3;

        // pipe!(10, f, g, h) = h(g(f(10))) = h(g(11)) = h(22) = 19
        let result = pipe!(10, f, g, h);
        assert_eq!(result, 19);
    }

    // ==================== curry! macro tests ====================

    #[test]
    fn test_curry2() {
        let add = |a: i32, b: i32| a + b;
        let curried = curry2!(add);

        let add5 = curried(5);
        assert_eq!(add5(10), 15);
        assert_eq!(add5(20), 25);
    }

    #[test]
    fn test_curry3() {
        let add3 = |a: i32, b: i32, c: i32| a + b + c;
        let curried = curry3!(add3);

        let partial1 = curried(1);
        let partial2 = partial1(2);
        assert_eq!(partial2(3), 6);
    }

    // ==================== flip! macro tests ====================

    #[test]
    fn test_flip() {
        let sub = |a: i32, b: i32| a - b;
        let flipped = flip!(sub);

        assert_eq!(sub(10, 3), 7);
        assert_eq!(flipped(10, 3), -7); // 3 - 10
    }

    // ==================== constant! macro tests ====================

    #[test]
    fn test_constant() {
        let always_42 = constant!(42);
        assert_eq!(always_42(100), 42);
        assert_eq!(always_42(0), 42);
    }
}
