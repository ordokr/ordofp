//! Aspectus (Lens) - Focus on a single field within a product type (struct).
//!
//! > *"Aspectus est actus intellectus quo rem intuetur."*
//! > — An aspect is the act of the intellect by which it beholds a thing. (Scholastic)
//!
//! An aspectus provides a way to:
//! - **Get** a value from a larger structure
//! - **Set** a value within a larger structure (returning a new structure)
//! - **Modify** a value using a function (returning a new structure)
//!
//! Aspectus are composable, allowing you to focus on deeply nested data.

/// An aspectus (lens) focusing on a value of type `A` within a structure of type `S`.
///
/// An aspectus is defined by two functions:
/// - `get: &S -> A` - Extract the focused value
/// - `set: (&S, A) -> S` - Set a new value, returning a new structure
///
/// # Type Parameters
/// - `S` - The source/whole type
/// - `A` - The target/part type
/// - `GetFn` - The getter function type
/// - `SetFn` - The setter function type
pub struct Aspectus<S, A, GetFn, SetFn>
where
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    get_fn: GetFn,
    set_fn: SetFn,
    _phantom: core::marker::PhantomData<fn(&S) -> A>,
}

impl<S, A, GetFn, SetFn> Clone for Aspectus<S, A, GetFn, SetFn>
where
    GetFn: Fn(&S) -> A + Clone,
    SetFn: Fn(&S, A) -> S + Clone,
{
    fn clone(&self) -> Self {
        Self {
            get_fn: self.get_fn.clone(),
            set_fn: self.set_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S, A, GetFn, SetFn> Aspectus<S, A, GetFn, SetFn>
where
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    /// Create a new aspectus from getter and setter functions.
    ///
    /// # Arguments
    /// - `get_fn` - Function to extract the focused value from the structure
    /// - `set_fn` - Function to set a new value in the structure
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::Aspectus;
    ///
    /// #[derive(Clone)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let x_aspectus = Aspectus::new(
    ///     |p: &Point| p.x,
    ///     |p: &Point, x: i32| Point { x, y: p.y },
    /// );
    ///
    /// let point = Point { x: 10, y: 20 };
    /// assert_eq!(x_aspectus.get(&point), 10);
    /// ```
    #[inline]
    pub fn new(get_fn: GetFn, set_fn: SetFn) -> Self {
        Self {
            get_fn,
            set_fn,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Extract the focused value from the structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aspectus;
    ///
    /// #[derive(Clone)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let x_aspectus = aspectus(
    ///     |p: &Point| p.x,
    ///     |p: &Point, x: i32| Point { x, y: p.y },
    /// );
    ///
    /// let point = Point { x: 10, y: 20 };
    /// assert_eq!(x_aspectus.get(&point), 10);
    /// ```
    #[inline]
    pub fn get(&self, source: &S) -> A {
        (self.get_fn)(source)
    }

    /// Set a new value for the focused part, returning a new structure.
    ///
    /// This does not modify the original structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aspectus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let x_aspectus = aspectus(
    ///     |p: &Point| p.x,
    ///     |p: &Point, x: i32| Point { x, y: p.y },
    /// );
    ///
    /// let point = Point { x: 10, y: 20 };
    /// let updated = x_aspectus.set(&point, 100);
    ///
    /// assert_eq!(updated, Point { x: 100, y: 20 });
    /// assert_eq!(point.x, 10); // Original unchanged
    /// ```
    #[inline]
    pub fn set(&self, source: &S, value: A) -> S {
        (self.set_fn)(source, value)
    }

    /// Modify the focused value using a function, returning a new structure.
    ///
    /// Equivalent to `aspectus.set(source, f(aspectus.get(source)))`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aspectus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let x_aspectus = aspectus(
    ///     |p: &Point| p.x,
    ///     |p: &Point, x: i32| Point { x, y: p.y },
    /// );
    ///
    /// let point = Point { x: 10, y: 20 };
    /// let doubled = x_aspectus.modify(&point, |x| x * 2);
    ///
    /// assert_eq!(doubled, Point { x: 20, y: 20 });
    /// ```
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(A) -> A,
    {
        let value = self.get(source);
        self.set(source, f(value))
    }

    /// Compose this aspectus with another aspectus to focus on nested data.
    ///
    /// If `self` focuses on `A` within `S`, and `other` focuses on `B` within `A`,
    /// the composed aspectus focuses on `B` within `S`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aspectus;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Address { street: String, city: String }
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Person { name: String, address: Address }
    ///
    /// let address_aspectus = aspectus(
    ///     |p: &Person| p.address.clone(),
    ///     |p: &Person, a: Address| Person { name: p.name.clone(), address: a },
    /// );
    ///
    /// let street_aspectus = aspectus(
    ///     |a: &Address| a.street.clone(),
    ///     |a: &Address, s: String| Address { street: s, city: a.city.clone() },
    /// );
    ///
    /// let person_street = address_aspectus.compose(&street_aspectus);
    ///
    /// let person = Person {
    ///     name: "Alice".to_string(),
    ///     address: Address {
    ///         street: "123 Main St".to_string(),
    ///         city: "Springfield".to_string(),
    ///     },
    /// };
    ///
    /// assert_eq!(person_street.get(&person), "123 Main St");
    /// ```
    #[inline]
    pub fn compose<B, GetFn2, SetFn2>(
        &self,
        other: &Aspectus<A, B, GetFn2, SetFn2>,
    ) -> ComposedAspectus<S, A, B, GetFn, SetFn, GetFn2, SetFn2>
    where
        GetFn: Clone,
        SetFn: Clone,
        GetFn2: Fn(&A) -> B + Clone,
        SetFn2: Fn(&A, B) -> A + Clone,
    {
        ComposedAspectus {
            outer: self.clone(),
            inner: other.clone(),
        }
    }
}

/// A composed aspectus focusing on `B` within `S` through an intermediate `A`.
#[derive(Clone)]
pub struct ComposedAspectus<S, A, B, GetFn1, SetFn1, GetFn2, SetFn2>
where
    GetFn1: Fn(&S) -> A,
    SetFn1: Fn(&S, A) -> S,
    GetFn2: Fn(&A) -> B,
    SetFn2: Fn(&A, B) -> A,
{
    outer: Aspectus<S, A, GetFn1, SetFn1>,
    inner: Aspectus<A, B, GetFn2, SetFn2>,
}

impl<S, A, B, GetFn1, SetFn1, GetFn2, SetFn2>
    ComposedAspectus<S, A, B, GetFn1, SetFn1, GetFn2, SetFn2>
where
    GetFn1: Fn(&S) -> A,
    SetFn1: Fn(&S, A) -> S,
    GetFn2: Fn(&A) -> B,
    SetFn2: Fn(&A, B) -> A,
{
    /// Extract the focused value from the structure.
    #[inline]
    pub fn get(&self, source: &S) -> B {
        let a = self.outer.get(source);
        self.inner.get(&a)
    }

    /// Set a new value for the focused part, returning a new structure.
    #[inline]
    pub fn set(&self, source: &S, value: B) -> S {
        let a = self.outer.get(source);
        let new_a = self.inner.set(&a, value);
        self.outer.set(source, new_a)
    }

    /// Modify the focused value using a function, returning a new structure.
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(B) -> B,
    {
        let a = self.outer.get(source);
        let b = self.inner.get(&a);
        let new_b = f(b);
        let new_a = self.inner.set(&a, new_b);
        self.outer.set(source, new_a)
    }
}

/// Create a new aspectus from getter and setter functions.
///
/// This is a convenience function equivalent to `Aspectus::new`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::aspectus;
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Point { x: i32, y: i32 }
///
/// let x_aspectus = aspectus(
///     |p: &Point| p.x,
///     |p: &Point, x: i32| Point { x, y: p.y },
/// );
///
/// let point = Point { x: 10, y: 20 };
/// assert_eq!(x_aspectus.get(&point), 10);
/// assert_eq!(x_aspectus.set(&point, 100), Point { x: 100, y: 20 });
/// ```
#[inline]
pub fn aspectus<S, A, GetFn, SetFn>(get_fn: GetFn, set_fn: SetFn) -> Aspectus<S, A, GetFn, SetFn>
where
    GetFn: Fn(&S) -> A,
    SetFn: Fn(&S, A) -> S,
{
    Aspectus::new(get_fn, set_fn)
}

/// An aspectus that works with references, avoiding cloning.
///
/// This is useful when you only need to read values and don't need to modify them.
pub struct AspectusRef<S, A, GetFn>
where
    GetFn: Fn(&S) -> &A,
{
    get_fn: GetFn,
    // `fn(&S) -> &A` is always `Clone`/`Copy`, so (unlike `PhantomData<(S, A)>`
    // with `#[derive(Clone)]`) cloning does not spuriously require `S: Clone`.
    _phantom: core::marker::PhantomData<fn(&S) -> &A>,
}

impl<S, A, GetFn> Clone for AspectusRef<S, A, GetFn>
where
    GetFn: Fn(&S) -> &A + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            get_fn: self.get_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S, A, GetFn> AspectusRef<S, A, GetFn>
where
    GetFn: Fn(&S) -> &A,
{
    /// Create a new reference aspectus.
    #[inline]
    pub fn new(get_fn: GetFn) -> Self {
        Self {
            get_fn,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get a reference to the focused value.
    #[inline]
    pub fn get<'a>(&self, source: &'a S) -> &'a A {
        (self.get_fn)(source)
    }

    /// Compose with another reference aspectus to focus on nested data, reading
    /// through both levels **without cloning** any intermediate.
    ///
    /// If `self` focuses on `&A` within `S`, and `other` focuses on `&B` within
    /// `A`, the composed reference aspectus reads `&B` from `&S` directly.
    /// Unlike the owning [`Aspectus::compose`], no intermediate `A` is cloned —
    /// the read is a pure pointer chase.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::AspectusRef;
    ///
    /// struct Course { name: String }
    /// struct Enrollment { course: Course }
    ///
    /// let course = AspectusRef::new(|e: &Enrollment| &e.course);
    /// let name = AspectusRef::new(|c: &Course| &c.name);
    /// let course_name = course.compose(&name);
    ///
    /// let e = Enrollment { course: Course { name: "FP".to_string() } };
    /// assert_eq!(course_name.get(&e), "FP"); // no clone of Course or name
    /// ```
    #[inline]
    pub fn compose<B, GetFn2>(
        &self,
        other: &AspectusRef<A, B, GetFn2>,
    ) -> ComposedAspectusRef<S, A, B, GetFn, GetFn2>
    where
        GetFn: Clone,
        GetFn2: Fn(&A) -> &B + Clone,
    {
        ComposedAspectusRef {
            outer: self.clone(),
            inner: other.clone(),
        }
    }
}

/// A composed reference aspectus focusing on `&B` within `S` through an
/// intermediate `A`, reading with **zero clones** (each level returns a borrow).
pub struct ComposedAspectusRef<S, A, B, GetFn1, GetFn2>
where
    GetFn1: Fn(&S) -> &A,
    GetFn2: Fn(&A) -> &B,
{
    outer: AspectusRef<S, A, GetFn1>,
    inner: AspectusRef<A, B, GetFn2>,
}

impl<S, A, B, GetFn1, GetFn2> Clone for ComposedAspectusRef<S, A, B, GetFn1, GetFn2>
where
    GetFn1: Fn(&S) -> &A + Clone,
    GetFn2: Fn(&A) -> &B + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            outer: self.outer.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<S, A, B, GetFn1, GetFn2> ComposedAspectusRef<S, A, B, GetFn1, GetFn2>
where
    GetFn1: Fn(&S) -> &A,
    GetFn2: Fn(&A) -> &B,
{
    /// Read a reference to the deeply-focused value without cloning.
    ///
    /// `A: 'a` simply states the intermediate outlives the borrow — always true,
    /// since the reference being returned is reached through a live `&'a A`.
    #[inline]
    pub fn get<'a>(&self, source: &'a S) -> &'a B
    where
        A: 'a,
    {
        self.inner.get(self.outer.get(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Line {
        start: Point,
        end: Point,
    }

    #[test]
    fn test_aspectus_basic() {
        let x_aspectus = aspectus(|p: &Point| p.x, |p: &Point, x: i32| Point { x, y: p.y });

        let point = Point { x: 10, y: 20 };
        assert_eq!(x_aspectus.get(&point), 10);
        assert_eq!(x_aspectus.set(&point, 100), Point { x: 100, y: 20 });
        assert_eq!(x_aspectus.modify(&point, |x| x + 5), Point { x: 15, y: 20 });
    }

    #[test]
    fn test_aspectus_composition() {
        let start_aspectus = aspectus(
            |l: &Line| l.start.clone(),
            |l: &Line, start: Point| Line {
                start,
                end: l.end.clone(),
            },
        );

        let x_aspectus = aspectus(|p: &Point| p.x, |p: &Point, x: i32| Point { x, y: p.y });

        let start_x = start_aspectus.compose(&x_aspectus);

        let line = Line {
            start: Point { x: 0, y: 0 },
            end: Point { x: 10, y: 10 },
        };

        assert_eq!(start_x.get(&line), 0);
        assert_eq!(
            start_x.set(&line, 5),
            Line {
                start: Point { x: 5, y: 0 },
                end: Point { x: 10, y: 10 },
            }
        );
    }

    #[test]
    fn test_aspectus_ref() {
        let x_ref = AspectusRef::new(|p: &Point| &p.x);

        let point = Point { x: 10, y: 20 };
        assert_eq!(*x_ref.get(&point), 10);
    }

    #[test]
    fn test_aspectus_ref_compose_zero_clone() {
        // Composed reference aspectus reads nested data without cloning the
        // intermediate (the whole point of F1). Line -> start (Point) -> x.
        let start_ref = AspectusRef::new(|l: &Line| &l.start);
        let x_ref = AspectusRef::new(|p: &Point| &p.x);
        let start_x = start_ref.compose(&x_ref);

        let line = Line {
            start: Point { x: 3, y: 7 },
            end: Point { x: 10, y: 10 },
        };

        // Reads return a borrow that points into `line` — proving no intermediate
        // Point was cloned (the address must be inside `line`).
        let got: &i32 = start_x.get(&line);
        assert_eq!(*got, 3);
        assert!(core::ptr::eq(got, &raw const line.start.x));
    }

    #[test]
    fn test_aspectus_ref_compose_is_clone() {
        // The composed reference aspectus is Clone (needed to store/reuse it),
        // and cloning does not require S/A/B: Clone.
        let start_ref = AspectusRef::new(|l: &Line| &l.start);
        let x_ref = AspectusRef::new(|p: &Point| &p.x);
        let start_x = start_ref.compose(&x_ref);
        let cloned = start_x.clone();

        let line = Line {
            start: Point { x: 9, y: 1 },
            end: Point { x: 2, y: 2 },
        };
        assert_eq!(*cloned.get(&line), 9);
    }

    #[test]
    fn test_composed_aspectus_modify() {
        // ComposedAspectus::modify follows a distinct code path from set/get and
        // was previously untested. Verify it correctly threads the transformation
        // through both lenses without touching unrelated fields.
        let start_aspectus = aspectus(
            |l: &Line| l.start.clone(),
            |l: &Line, start: Point| Line {
                start,
                end: l.end.clone(),
            },
        );
        let x_aspectus = aspectus(|p: &Point| p.x, |p: &Point, x: i32| Point { x, y: p.y });
        let start_x = start_aspectus.compose(&x_aspectus);

        let line = Line {
            start: Point { x: 3, y: 7 },
            end: Point { x: 10, y: 10 },
        };

        let result = start_x.modify(&line, |x| x * 2);

        assert_eq!(result.start.x, 6, "focused field should be doubled");
        assert_eq!(
            result.start.y, 7,
            "non-focused field in inner struct must be unchanged"
        );
        assert_eq!(
            result.end, line.end,
            "non-focused field in outer struct must be unchanged"
        );
    }
}
