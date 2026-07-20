//! # Optics Laws
//!
//! Property-based laws for [`Aspectus`] (lens) and [`Divisio`] (prism).
//!
//! ## Lens laws
//!
//! - **GetPut**: `set(s, get(s)) == s` — writing back what you read changes nothing.
//! - **PutGet**: `get(set(s, a)) == a` — you read what you wrote.
//! - **PutPut**: `set(set(s, a1), a2) == set(s, a2)` — the last write wins.
//!
//! ## Prism laws
//!
//! - **PreviewReview**: `preview(review(a)) == Some(a)` — a built variant matches itself.
//! - **ReviewPreview**: if `preview(s) == Some(a)` then `review(a) == s` — a match
//!   loses no information beyond the variant tag.
//!
//! ## Usage
//!
//! ```ignore
//! use ordofp_laws::optics_laws::{lens_get_put, lens_put_get, lens_put_put};
//! use quickcheck::quickcheck;
//!
//! quickcheck(|x: i32, y: i32, a: i32| {
//!     let lens = ordofp::optics::aspectus(
//!         |p: &(i32, i32)| p.0,
//!         |p: &(i32, i32), v: i32| (v, p.1),
//!     );
//!     lens_get_put(&lens, &(x, y)) && lens_put_get(&lens, &(x, y), a)
//! });
//! ```

use ordofp::optics::{Aspectus, Divisio};

/// **GetPut law**: writing back the value you just read is a no-op.
///
/// ```text
/// set(s, get(s)) == s
/// ```
pub fn lens_get_put<S, A, GetFn, SetFn>(lens: &Aspectus<S, A, GetFn, SetFn>, s: &S) -> bool
where
    S: PartialEq,
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    lens.set(s, lens.get(s)) == *s
}

/// **PutGet law**: you read exactly what you wrote.
///
/// ```text
/// get(set(s, a)) == a
/// ```
pub fn lens_put_get<S, A, GetFn, SetFn>(lens: &Aspectus<S, A, GetFn, SetFn>, s: &S, a: A) -> bool
where
    A: Clone + PartialEq,
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    lens.get(&lens.set(s, a.clone())) == a
}

/// **PutPut law**: the second write completely overwrites the first.
///
/// ```text
/// set(set(s, a1), a2) == set(s, a2)
/// ```
pub fn lens_put_put<S, A, GetFn, SetFn>(
    lens: &Aspectus<S, A, GetFn, SetFn>,
    s: &S,
    a1: A,
    a2: A,
) -> bool
where
    S: PartialEq,
    A: Clone,
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    lens.set(&lens.set(s, a1), a2.clone()) == lens.set(s, a2)
}

/// **PreviewReview law**: a freshly built variant matches itself.
///
/// ```text
/// preview(review(a)) == Some(a)
/// ```
pub fn prism_preview_review<S, A, PreviewFn, ReviewFn>(
    prism: &Divisio<S, A, PreviewFn, ReviewFn>,
    a: A,
) -> bool
where
    A: Clone + PartialEq,
    PreviewFn: Fn(&S) -> Option<A>,
    ReviewFn: Fn(A) -> S,
{
    prism.preview(&prism.review(a.clone())) == Some(a)
}

/// **ReviewPreview law**: matching then rebuilding reproduces the source.
///
/// Holds vacuously when the prism does not match `s`.
///
/// ```text
/// preview(s) == Some(a)  ==>  review(a) == s
/// ```
pub fn prism_review_preview<S, A, PreviewFn, ReviewFn>(
    prism: &Divisio<S, A, PreviewFn, ReviewFn>,
    s: &S,
) -> bool
where
    S: PartialEq,
    PreviewFn: Fn(&S) -> Option<A>,
    ReviewFn: Fn(A) -> S,
{
    match prism.preview(s) {
        Some(a) => prism.review(a) == *s,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordofp::optics::{aspectus, divisio};
    use quickcheck::quickcheck;

    #[derive(Clone, Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    fn x_lens() -> Aspectus<Point, i32, impl Fn(&Point) -> i32, impl Fn(&Point, i32) -> Point> {
        aspectus(|p: &Point| p.x, |p: &Point, x: i32| Point { x, y: p.y })
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Shape {
        Circle(i64),
        Rectangle(i64, i64),
    }

    fn circle_prism() -> Divisio<Shape, i64, impl Fn(&Shape) -> Option<i64>, impl Fn(i64) -> Shape>
    {
        divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                Shape::Rectangle(..) => None,
            },
            Shape::Circle,
        )
    }

    quickcheck! {
        fn point_x_lens_get_put(x: i32, y: i32) -> bool {
            lens_get_put(&x_lens(), &Point { x, y })
        }

        fn point_x_lens_put_get(x: i32, y: i32, a: i32) -> bool {
            lens_put_get(&x_lens(), &Point { x, y }, a)
        }

        fn point_x_lens_put_put(x: i32, y: i32, a1: i32, a2: i32) -> bool {
            lens_put_put(&x_lens(), &Point { x, y }, a1, a2)
        }

        fn circle_prism_preview_review(r: i64) -> bool {
            prism_preview_review(&circle_prism(), r)
        }

        fn circle_prism_review_preview(circle: bool, a: i64, b: i64) -> bool {
            let shape = if circle { Shape::Circle(a) } else { Shape::Rectangle(a, b) };
            prism_review_preview(&circle_prism(), &shape)
        }
    }

    /// An unlawful lens (setter ignores the value) must be caught by PutGet —
    /// the laws are only worth shipping if they can actually fail.
    #[test]
    fn unlawful_lens_fails_put_get() {
        let broken = aspectus(|p: &Point| p.x, |p: &Point, _x: i32| p.clone());
        assert!(!lens_put_get(&broken, &Point { x: 1, y: 2 }, 42));
    }
}
