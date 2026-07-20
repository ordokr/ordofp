use ordofp::typeclasses::Category;
use ordofp_laws::category_laws;

struct MulCat;

// A category where objects are ignored and morphisms are integers.
// Composition is multiplication. Identity is 1.
impl Category for MulCat {
    type Hom<A, B> = i32;

    fn id<A>() -> Self::Hom<A, A> {
        1
    }

    fn compose<A, B, C>(f: Self::Hom<B, C>, g: Self::Hom<A, B>) -> Self::Hom<A, C> {
        f * g
    }
}

#[test]
fn test_mul_cat_identity() {
    // 1 * 5 == 5 AND 5 * 1 == 5
    assert!(category_laws::identity::<MulCat, (), ()>(5));
}

#[test]
fn test_mul_cat_associativity() {
    // (4 * 3) * 2 == 4 * (3 * 2)
    assert!(category_laws::associativity::<MulCat, (), (), (), ()>(
        2, 3, 4
    ));
}
