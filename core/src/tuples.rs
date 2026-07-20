//! This module is held separate to put generated variadic generics for tuples
//! at the end of the documentation so as to not disturb the reader when reading
//! documentation.
//!
//! ```
//! # use ordofp_core::hlist::*;
//! # use ordofp_core::{hlist, HList};
//! # fn main() {
//! let h = hlist![ 42f32, true, "hello" ];
//! let t: (f32, bool, &str) = h.into();
//! assert_eq!(t, (42f32, true, "hello"));
//!
//! let t2 = (999, false, "world");
//! let h2: HList![ isize, bool, &str ] = t2.into();
//! assert_eq!(h2, hlist![ 999, false, "world" ]);
//! # }
//! ```

use crate::universalis::Universalis;

// Individual tuple implementations using field access to avoid naming convention conflicts
impl<A> Universalis for (A,) {
    type Repr = HList![A];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        (r.head,)
    }
}
impl<A> From<(A,)> for HList![A] {
    #[inline]
    fn from(tup: (A,)) -> Self {
        Universalis::into(tup)
    }
}
impl<A> From<HList![A]> for (A,) {
    #[inline]
    fn from(h: HList![A]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B> Universalis for (A, B) {
    type Repr = HList![A, B];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0, self.1]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b] = r;
        (a, b)
    }
}
impl<A, B> From<(A, B)> for HList![A, B] {
    #[inline]
    fn from(tup: (A, B)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B> From<HList![A, B]> for (A, B) {
    #[inline]
    fn from(h: HList![A, B]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C> Universalis for (A, B, C) {
    type Repr = HList![A, B, C];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0, self.1, self.2]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c] = r;
        (a, b, c)
    }
}
impl<A, B, C> From<(A, B, C)> for HList![A, B, C] {
    #[inline]
    fn from(tup: (A, B, C)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C> From<HList![A, B, C]> for (A, B, C) {
    #[inline]
    fn from(h: HList![A, B, C]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D> Universalis for (A, B, C, D) {
    type Repr = HList![A, B, C, D];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0, self.1, self.2, self.3]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d] = r;
        (a, b, c, d)
    }
}
impl<A, B, C, D> From<(A, B, C, D)> for HList![A, B, C, D] {
    #[inline]
    fn from(tup: (A, B, C, D)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D> From<HList![A, B, C, D]> for (A, B, C, D) {
    #[inline]
    fn from(h: HList![A, B, C, D]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E> Universalis for (A, B, C, D, E) {
    type Repr = HList![A, B, C, D, E];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0, self.1, self.2, self.3, self.4]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e] = r;
        (a, b, c, d, e)
    }
}
impl<A, B, C, D, E> From<(A, B, C, D, E)> for HList![A, B, C, D, E] {
    #[inline]
    fn from(tup: (A, B, C, D, E)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E> From<HList![A, B, C, D, E]> for (A, B, C, D, E) {
    #[inline]
    fn from(h: HList![A, B, C, D, E]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F> Universalis for (A, B, C, D, E, F) {
    type Repr = HList![A, B, C, D, E, F];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0, self.1, self.2, self.3, self.4, self.5]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f] = r;
        (a, b, c, d, e, f)
    }
}
impl<A, B, C, D, E, F> From<(A, B, C, D, E, F)> for HList![A, B, C, D, E, F] {
    #[inline]
    fn from(tup: (A, B, C, D, E, F)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F> From<HList![A, B, C, D, E, F]> for (A, B, C, D, E, F) {
    #[inline]
    fn from(h: HList![A, B, C, D, E, F]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F, G> Universalis for (A, B, C, D, E, F, G) {
    type Repr = HList![A, B, C, D, E, F, G];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![self.0, self.1, self.2, self.3, self.4, self.5, self.6]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f, g] = r;
        (a, b, c, d, e, f, g)
    }
}
impl<A, B, C, D, E, F, G> From<(A, B, C, D, E, F, G)> for HList![A, B, C, D, E, F, G] {
    #[inline]
    fn from(tup: (A, B, C, D, E, F, G)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F, G> From<HList![A, B, C, D, E, F, G]> for (A, B, C, D, E, F, G) {
    #[inline]
    fn from(h: HList![A, B, C, D, E, F, G]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F, G, H> Universalis for (A, B, C, D, E, F, G, H) {
    type Repr = HList![A, B, C, D, E, F, G, H];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7
        ]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f, g, h] = r;
        (a, b, c, d, e, f, g, h)
    }
}
impl<A, B, C, D, E, F, G, H> From<(A, B, C, D, E, F, G, H)> for HList![A, B, C, D, E, F, G, H] {
    #[inline]
    fn from(tup: (A, B, C, D, E, F, G, H)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F, G, H> From<HList![A, B, C, D, E, F, G, H]> for (A, B, C, D, E, F, G, H) {
    #[inline]
    fn from(h: HList![A, B, C, D, E, F, G, H]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F, G, H, I> Universalis for (A, B, C, D, E, F, G, H, I) {
    type Repr = HList![A, B, C, D, E, F, G, H, I];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8
        ]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f, g, h, i] = r;
        (a, b, c, d, e, f, g, h, i)
    }
}
impl<A, B, C, D, E, F, G, H, I> From<(A, B, C, D, E, F, G, H, I)>
    for HList![A, B, C, D, E, F, G, H, I]
{
    #[inline]
    fn from(tup: (A, B, C, D, E, F, G, H, I)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F, G, H, I> From<HList![A, B, C, D, E, F, G, H, I]>
    for (A, B, C, D, E, F, G, H, I)
{
    #[inline]
    fn from(h: HList![A, B, C, D, E, F, G, H, I]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F, G, H, I, J> Universalis for (A, B, C, D, E, F, G, H, I, J) {
    type Repr = HList![A, B, C, D, E, F, G, H, I, J];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9
        ]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f, g, h, i, j] = r;
        (a, b, c, d, e, f, g, h, i, j)
    }
}
impl<A, B, C, D, E, F, G, H, I, J> From<(A, B, C, D, E, F, G, H, I, J)>
    for HList![A, B, C, D, E, F, G, H, I, J]
{
    #[inline]
    fn from(tup: (A, B, C, D, E, F, G, H, I, J)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F, G, H, I, J> From<HList![A, B, C, D, E, F, G, H, I, J]>
    for (A, B, C, D, E, F, G, H, I, J)
{
    #[inline]
    fn from(h: HList![A, B, C, D, E, F, G, H, I, J]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F, G, H, I, J, K> Universalis for (A, B, C, D, E, F, G, H, I, J, K) {
    type Repr = HList![A, B, C, D, E, F, G, H, I, J, K];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9, self.10
        ]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f, g, h, i, j, k] = r;
        (a, b, c, d, e, f, g, h, i, j, k)
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K> From<(A, B, C, D, E, F, G, H, I, J, K)>
    for HList![A, B, C, D, E, F, G, H, I, J, K]
{
    #[inline]
    fn from(tup: (A, B, C, D, E, F, G, H, I, J, K)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K> From<HList![A, B, C, D, E, F, G, H, I, J, K]>
    for (A, B, C, D, E, F, G, H, I, J, K)
{
    #[inline]
    fn from(h: HList![A, B, C, D, E, F, G, H, I, J, K]) -> Self {
        Universalis::from(h)
    }
}

impl<A, B, C, D, E, F, G, H, I, J, K, L> Universalis for (A, B, C, D, E, F, G, H, I, J, K, L) {
    type Repr = HList![A, B, C, D, E, F, G, H, I, J, K, L];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9,
            self.10, self.11
        ]
    }
    #[inline]
    fn from(r: Self::Repr) -> Self {
        let coniunctio_pat![a, b, c, d, e, f, g, h, i, j, k, l] = r;
        (a, b, c, d, e, f, g, h, i, j, k, l)
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L> From<(A, B, C, D, E, F, G, H, I, J, K, L)>
    for HList![A, B, C, D, E, F, G, H, I, J, K, L]
{
    #[inline]
    fn from(tup: (A, B, C, D, E, F, G, H, I, J, K, L)) -> Self {
        Universalis::into(tup)
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L> From<HList![A, B, C, D, E, F, G, H, I, J, K, L]>
    for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    #[inline]
    fn from(h: HList![A, B, C, D, E, F, G, H, I, J, K, L]) -> Self {
        Universalis::from(h)
    }
}

impl Universalis for () {
    type Repr = HList![];
    #[inline]
    fn into(self) -> Self::Repr {
        hlist![]
    }
    #[inline]
    fn from(_: Self::Repr) -> Self {}
}

impl From<()> for HList![] {
    #[inline]
    fn from((): ()) -> Self {
        hlist![]
    }
}
