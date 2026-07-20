//! Divisio (Prism) - Focus on a single variant within a sum type (enum).
//!
//! > *"Divisio est distributio generis in species."*
//! > — Division is the distribution of a genus into species. (Scholastic)
//!
//! A divisio provides a way to:
//! - **Preview** a value (returns `Some` if the variant matches, `None` otherwise)
//! - **Review** a value (construct the variant from the inner value)
//! - **Modify** a value if the variant matches
//!
//! Divisio are composable, allowing you to focus on nested enum variants.

/// A divisio (prism) focusing on a value of type `A` within a sum type `S`.
///
/// A divisio is defined by two functions:
/// - `preview: &S -> Option<A>` - Extract the value if the variant matches
/// - `review: A -> S` - Construct the variant from a value
///
/// # Type Parameters
/// - `S` - The source/whole sum type
/// - `A` - The target/part type (the variant's inner value)
/// - `PreviewFn` - The preview function type
/// - `ReviewFn` - The review function type
pub struct Divisio<S, A, PreviewFn, ReviewFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    ReviewFn: Fn(A) -> S,
{
    preview_fn: PreviewFn,
    review_fn: ReviewFn,
    _phantom: core::marker::PhantomData<fn(&S) -> Option<A>>,
}

impl<S, A, PreviewFn, ReviewFn> Clone for Divisio<S, A, PreviewFn, ReviewFn>
where
    PreviewFn: Fn(&S) -> Option<A> + Clone,
    ReviewFn: Fn(A) -> S + Clone,
{
    fn clone(&self) -> Self {
        Self {
            preview_fn: self.preview_fn.clone(),
            review_fn: self.review_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S, A, PreviewFn, ReviewFn> Divisio<S, A, PreviewFn, ReviewFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    ReviewFn: Fn(A) -> S,
{
    /// Create a new divisio from preview and review functions.
    ///
    /// # Arguments
    /// - `preview_fn` - Function to extract the value if the variant matches
    /// - `review_fn` - Function to construct the variant from a value
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::Divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Either<L, R> {
    ///     Left(L),
    ///     Right(R),
    /// }
    ///
    /// let left_divisio: Divisio<Either<i32, String>, i32, _, _> = Divisio::new(
    ///     |e| match e {
    ///         Either::Left(l) => Some(*l),
    ///         _ => None,
    ///     },
    ///     Either::Left,
    /// );
    ///
    /// let left: Either<i32, String> = Either::Left(42);
    /// assert_eq!(left_divisio.preview(&left), Some(42));
    /// ```
    #[inline]
    pub fn new(preview_fn: PreviewFn, review_fn: ReviewFn) -> Self {
        Self {
            preview_fn,
            review_fn,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Attempt to extract the focused value from the sum type.
    ///
    /// Returns `Some(value)` if the variant matches, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Shape {
    ///     Circle(f64),
    ///     Rectangle(f64, f64),
    /// }
    ///
    /// let circle_divisio = divisio(
    ///     |s: &Shape| match s {
    ///         Shape::Circle(r) => Some(*r),
    ///         _ => None,
    ///     },
    ///     Shape::Circle,
    /// );
    ///
    /// assert_eq!(circle_divisio.preview(&Shape::Circle(5.0)), Some(5.0));
    /// assert_eq!(circle_divisio.preview(&Shape::Rectangle(3.0, 4.0)), None);
    /// ```
    #[inline]
    pub fn preview(&self, source: &S) -> Option<A> {
        (self.preview_fn)(source)
    }

    /// Construct the variant from a value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Shape {
    ///     Circle(f64),
    ///     Rectangle(f64, f64),
    /// }
    ///
    /// let circle_divisio = divisio(
    ///     |s: &Shape| match s {
    ///         Shape::Circle(r) => Some(*r),
    ///         _ => None,
    ///     },
    ///     Shape::Circle,
    /// );
    ///
    /// assert_eq!(circle_divisio.review(10.0), Shape::Circle(10.0));
    /// ```
    #[inline]
    pub fn review(&self, value: A) -> S {
        (self.review_fn)(value)
    }

    /// Modify the focused value if the variant matches.
    ///
    /// Returns `Some(new_value)` if the variant matched and was modified,
    /// `None` if the variant didn't match.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Shape {
    ///     Circle(f64),
    ///     Rectangle(f64, f64),
    /// }
    ///
    /// let circle_divisio = divisio(
    ///     |s: &Shape| match s {
    ///         Shape::Circle(r) => Some(*r),
    ///         _ => None,
    ///     },
    ///     Shape::Circle,
    /// );
    ///
    /// let circle = Shape::Circle(5.0);
    /// let doubled = circle_divisio.modify(&circle, |r| r * 2.0);
    /// assert_eq!(doubled, Some(Shape::Circle(10.0)));
    ///
    /// let rect = Shape::Rectangle(3.0, 4.0);
    /// let result = circle_divisio.modify(&rect, |r| r * 2.0);
    /// assert_eq!(result, None);
    /// ```
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> Option<S>
    where
        F: FnOnce(A) -> A,
    {
        self.preview(source).map(|value| self.review(f(value)))
    }

    /// Modify the focused value, or return the original if the variant doesn't match.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Shape {
    ///     Circle(f64),
    ///     Rectangle(f64, f64),
    /// }
    ///
    /// let circle_divisio = divisio(
    ///     |s: &Shape| match s {
    ///         Shape::Circle(r) => Some(*r),
    ///         _ => None,
    ///     },
    ///     Shape::Circle,
    /// );
    ///
    /// let circle = Shape::Circle(5.0);
    /// let doubled = circle_divisio.modify_or_identity(&circle, |r| r * 2.0);
    /// assert_eq!(doubled, Shape::Circle(10.0));
    ///
    /// let rect = Shape::Rectangle(3.0, 4.0);
    /// let result = circle_divisio.modify_or_identity(&rect, |r| r * 2.0);
    /// assert_eq!(result, Shape::Rectangle(3.0, 4.0)); // Unchanged
    /// ```
    #[inline]
    pub fn modify_or_identity<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(A) -> A,
        S: Clone,
    {
        self.modify(source, f).unwrap_or_else(|| source.clone())
    }

    /// Set a new value if the variant matches.
    ///
    /// Returns `Some(new_value)` if the variant matched,
    /// `None` if the variant didn't match.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Shape {
    ///     Circle(f64),
    ///     Rectangle(f64, f64),
    /// }
    ///
    /// let circle_divisio = divisio(
    ///     |s: &Shape| match s {
    ///         Shape::Circle(r) => Some(*r),
    ///         _ => None,
    ///     },
    ///     Shape::Circle,
    /// );
    ///
    /// let circle = Shape::Circle(5.0);
    /// let updated = circle_divisio.set_if_matches(&circle, 100.0);
    /// assert_eq!(updated, Some(Shape::Circle(100.0)));
    /// ```
    #[inline]
    pub fn set_if_matches(&self, source: &S, value: A) -> Option<S> {
        if self.preview(source).is_some() {
            Some(self.review(value))
        } else {
            None
        }
    }

    /// Check if the variant matches.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Shape {
    ///     Circle(f64),
    ///     Rectangle(f64, f64),
    /// }
    ///
    /// let circle_divisio = divisio(
    ///     |s: &Shape| match s {
    ///         Shape::Circle(r) => Some(*r),
    ///         _ => None,
    ///     },
    ///     Shape::Circle,
    /// );
    ///
    /// assert!(circle_divisio.matches(&Shape::Circle(5.0)));
    /// assert!(!circle_divisio.matches(&Shape::Rectangle(3.0, 4.0)));
    /// ```
    #[inline]
    pub fn matches(&self, source: &S) -> bool {
        self.preview(source).is_some()
    }

    /// Compose this divisio with another divisio to focus on nested variants.
    ///
    /// If `self` focuses on `A` within `S`, and `other` focuses on `B` within `A`,
    /// the composed divisio focuses on `B` within `S`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::divisio;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Outer {
    ///     A(Inner),
    ///     B,
    /// }
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum Inner {
    ///     X(i32),
    ///     Y,
    /// }
    ///
    /// let a_divisio = divisio(
    ///     |o: &Outer| match o {
    ///         Outer::A(i) => Some(i.clone()),
    ///         _ => None,
    ///     },
    ///     Outer::A,
    /// );
    ///
    /// let x_divisio = divisio(
    ///     |i: &Inner| match i {
    ///         Inner::X(n) => Some(*n),
    ///         _ => None,
    ///     },
    ///     Inner::X,
    /// );
    ///
    /// let a_x = a_divisio.compose(&x_divisio);
    ///
    /// let outer = Outer::A(Inner::X(42));
    /// assert_eq!(a_x.preview(&outer), Some(42));
    ///
    /// let outer_b = Outer::B;
    /// assert_eq!(a_x.preview(&outer_b), None);
    /// ```
    #[inline]
    pub fn compose<B, PreviewFn2, ReviewFn2>(
        &self,
        other: &Divisio<A, B, PreviewFn2, ReviewFn2>,
    ) -> ComposedDivisio<S, A, B, PreviewFn, ReviewFn, PreviewFn2, ReviewFn2>
    where
        PreviewFn: Clone,
        ReviewFn: Clone,
        PreviewFn2: Fn(&A) -> Option<B> + Clone,
        ReviewFn2: Fn(B) -> A + Clone,
    {
        ComposedDivisio {
            outer: self.clone(),
            inner: other.clone(),
        }
    }
}

/// A composed divisio focusing on `B` within `S` through an intermediate `A`.
#[derive(Clone)]
pub struct ComposedDivisio<S, A, B, PreviewFn1, ReviewFn1, PreviewFn2, ReviewFn2>
where
    PreviewFn1: Fn(&S) -> Option<A>,
    ReviewFn1: Fn(A) -> S,
    PreviewFn2: Fn(&A) -> Option<B>,
    ReviewFn2: Fn(B) -> A,
{
    outer: Divisio<S, A, PreviewFn1, ReviewFn1>,
    inner: Divisio<A, B, PreviewFn2, ReviewFn2>,
}

impl<S, A, B, PreviewFn1, ReviewFn1, PreviewFn2, ReviewFn2>
    ComposedDivisio<S, A, B, PreviewFn1, ReviewFn1, PreviewFn2, ReviewFn2>
where
    PreviewFn1: Fn(&S) -> Option<A>,
    ReviewFn1: Fn(A) -> S,
    PreviewFn2: Fn(&A) -> Option<B>,
    ReviewFn2: Fn(B) -> A,
{
    /// Attempt to extract the focused value.
    #[inline]
    pub fn preview(&self, source: &S) -> Option<B> {
        self.outer
            .preview(source)
            .and_then(|a| self.inner.preview(&a))
    }

    /// Construct the nested variant from a value.
    #[inline]
    pub fn review(&self, value: B) -> S {
        self.outer.review(self.inner.review(value))
    }

    /// Modify the focused value if both variants match.
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> Option<S>
    where
        F: FnOnce(B) -> B,
    {
        self.preview(source).map(|value| self.review(f(value)))
    }

    /// Check if both variants match.
    #[inline]
    pub fn matches(&self, source: &S) -> bool {
        self.preview(source).is_some()
    }
}

/// Create a new divisio from preview and review functions.
///
/// This is a convenience function equivalent to `Divisio::new`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::divisio;
///
/// #[derive(Clone, Debug, PartialEq)]
/// enum Option2<T> {
///     Some(T),
///     None,
/// }
///
/// let some_divisio = divisio(
///     |o: &Option2<i32>| match o {
///         Option2::Some(x) => Some(*x),
///         _ => None,
///     },
///     Option2::Some,
/// );
///
/// assert_eq!(some_divisio.preview(&Option2::Some(42)), Some(42));
/// assert_eq!(some_divisio.preview(&Option2::None), None);
/// assert_eq!(some_divisio.review(100), Option2::Some(100));
/// ```
#[inline]
pub fn divisio<S, A, PreviewFn, ReviewFn>(
    preview_fn: PreviewFn,
    review_fn: ReviewFn,
) -> Divisio<S, A, PreviewFn, ReviewFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    ReviewFn: Fn(A) -> S,
{
    Divisio::new(preview_fn, review_fn)
}

/// A divisio that returns references instead of cloning.
///
/// This is useful when you only need to inspect values.
pub struct DivisioRef<S, A, PreviewFn>
where
    PreviewFn: Fn(&S) -> Option<&A>,
{
    preview_fn: PreviewFn,
    // `fn(&S) -> Option<&A>` is always `Clone`/`Copy`, so (unlike
    // `PhantomData<(S, A)>` with `#[derive(Clone)]`) cloning does not
    // spuriously require `S: Clone`.
    _phantom: core::marker::PhantomData<fn(&S) -> Option<&A>>,
}

impl<S, A, PreviewFn> Clone for DivisioRef<S, A, PreviewFn>
where
    PreviewFn: Fn(&S) -> Option<&A> + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            preview_fn: self.preview_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S, A, PreviewFn> DivisioRef<S, A, PreviewFn>
where
    PreviewFn: Fn(&S) -> Option<&A>,
{
    /// Create a new reference divisio.
    #[inline]
    pub fn new(preview_fn: PreviewFn) -> Self {
        Self {
            preview_fn,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get a reference to the focused value if the variant matches.
    #[inline]
    pub fn preview<'a>(&self, source: &'a S) -> Option<&'a A> {
        (self.preview_fn)(source)
    }

    /// Check if the variant matches.
    #[inline]
    pub fn matches(&self, source: &S) -> bool {
        self.preview(source).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::string::{String, ToString};

    #[derive(Clone, Debug, PartialEq)]
    enum Either<L, R> {
        Left(L),
        Right(R),
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Nested {
        A(Either<i32, String>),
        B,
    }

    #[test]
    fn test_divisio_basic() {
        let left_divisio: Divisio<Either<i32, String>, i32, _, _> = divisio(
            |e| match e {
                Either::Left(l) => Some(*l),
                _ => None,
            },
            Either::Left,
        );

        let left: Either<i32, String> = Either::Left(42);
        let right: Either<i32, String> = Either::Right("hello".to_string());

        assert_eq!(left_divisio.preview(&left), Some(42));
        assert_eq!(left_divisio.preview(&right), None);
        assert_eq!(left_divisio.review(100), Either::Left(100));
    }

    #[test]
    fn test_divisio_modify() {
        let left_divisio: Divisio<Either<i32, String>, i32, _, _> = divisio(
            |e| match e {
                Either::Left(l) => Some(*l),
                _ => None,
            },
            Either::Left,
        );

        let left: Either<i32, String> = Either::Left(42);
        let right: Either<i32, String> = Either::Right("hello".to_string());

        assert_eq!(
            left_divisio.modify(&left, |x| x * 2),
            Some(Either::Left(84))
        );
        assert_eq!(left_divisio.modify(&right, |x| x * 2), None);
    }

    #[test]
    fn test_divisio_composition() {
        let a_divisio: Divisio<Nested, Either<i32, String>, _, _> = divisio(
            |n| match n {
                Nested::A(e) => Some(e.clone()),
                _ => None,
            },
            Nested::A,
        );

        let left_divisio: Divisio<Either<i32, String>, i32, _, _> = divisio(
            |e| match e {
                Either::Left(l) => Some(*l),
                _ => None,
            },
            Either::Left,
        );

        let composed = a_divisio.compose(&left_divisio);

        let nested_left = Nested::A(Either::Left(42));
        let nested_right = Nested::A(Either::Right("hello".to_string()));
        let nested_b = Nested::B;

        assert_eq!(composed.preview(&nested_left), Some(42));
        assert_eq!(composed.preview(&nested_right), None);
        assert_eq!(composed.preview(&nested_b), None);
        assert_eq!(composed.review(100), Nested::A(Either::Left(100)));
    }

    #[test]
    fn test_divisio_ref() {
        let left_ref: DivisioRef<Either<i32, String>, i32, _> = DivisioRef::new(|e| match e {
            Either::Left(l) => Some(l),
            _ => None,
        });

        let left: Either<i32, String> = Either::Left(42);
        let right: Either<i32, String> = Either::Right("hello".to_string());

        assert_eq!(left_ref.preview(&left), Some(&42));
        assert_eq!(left_ref.preview(&right), None);
        assert!(left_ref.matches(&left));
        assert!(!left_ref.matches(&right));
    }
}
