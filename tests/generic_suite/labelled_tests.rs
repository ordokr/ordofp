// Test names embed Latin type names (Universalis, etc.) by project convention.

use ordofp::hlist::Sculptor;
use ordofp::labelled::Field;
use ordofp::labelled::Transfigurator;
use ordofp::labelled::chars::{
    La, Lc, Ld, Le, Lf, Lg, Li, Ll, Lm, Ln, Lr, Ls, Lt, Lx, Ly, N0, Ua, Ub, Uc, Underscore, Uv,
};
use ordofp::{Coniunctio, Disiunctio, NominataUniversalis, transform_from};
use ordofp::{from_labelled_universalis, into_labelled_universalis};
use ordofp_core::{field, hlist};
use time::OffsetDateTime;

#[derive(NominataUniversalis, Debug, PartialEq, Eq, Clone)]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[derive(NominataUniversalis)]
pub struct HasKeyword1 {
    pub r#type: i32,
}

#[derive(NominataUniversalis)]
pub struct HasKeyword2 {
    pub r#type: i32,
}

#[derive(NominataUniversalis)]
pub struct HasKeyword1Embedder {
    pub r#true: HasKeyword1,
}

#[derive(NominataUniversalis)]
pub struct HasKeyword2Embedder {
    pub r#true: HasKeyword2,
}

#[derive(NominataUniversalis)]
pub struct NormalUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

impl<'a> NormalUser<'a> {
    /// Helper function for building a `NormalUser`
    pub fn build() -> NormalUser<'a> {
        NormalUser {
            first_name: "Moe",
            last_name: "Ali",
            age: 30,
        }
    }
}

// Fields are jumbled :(
#[derive(NominataUniversalis)]
pub struct JumbledUser<'a> {
    pub last_name: &'a str,
    pub age: usize,
    pub first_name: &'a str,
}

#[derive(NominataUniversalis)]
pub struct NormalUserWithAudit<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
    pub created_at: OffsetDateTime,
}

#[derive(NominataUniversalis)]
pub struct JumbledUserWithAudit<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
    pub created_at: OffsetDateTime,
}

// Nested + Jumbled

#[derive(NominataUniversalis)]
pub struct InternalPhoneNumber {
    pub emergency: Option<usize>,
    pub main: usize,
    pub secondary: Option<usize>,
}

#[derive(NominataUniversalis)]
pub struct InternalAddress<'a> {
    pub is_whitelisted: bool,
    pub name: &'a str,
    pub phone: InternalPhoneNumber,
}

#[derive(NominataUniversalis)]
pub struct InternalUser<'a> {
    pub name: &'a str,
    pub age: usize,
    pub address: InternalAddress<'a>,
    pub is_banned: bool,
    pub あ: bool,
}

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub struct ExternalPhoneNumber {
    pub main: usize,
}

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub struct ExternalAddress<'a> {
    pub name: &'a str,
    pub phone: ExternalPhoneNumber,
}

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub struct ExternalUser<'a> {
    pub age: usize,
    pub address: ExternalAddress<'a>,
    pub name: &'a str,
    pub あ: bool,
}

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub struct TypeWrapper(pub String);

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub struct TypeWrapper2(pub String);

#[derive(NominataUniversalis, PartialEq, Debug)]
pub struct Vec4f(pub f32, pub f32, pub f32, pub f32);

#[derive(NominataUniversalis, PartialEq, Debug)]
pub struct Vec3f(pub f32, pub f32, pub f32);

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub enum LabelledEnum1 {
    VariantA,
    VariantB(i32),
    VariantC { x: String, y: bool },
}

#[derive(NominataUniversalis, PartialEq, Eq, Debug)]
pub enum LabelledEnum2 {
    VariantA,
    VariantC { x: String, y: bool },
    VariantB(i32),
}

#[test]
fn test_struct_from_labelled_universalis() {
    let h = hlist![
        field!((Lf, Li, Lr, Ls, Lt, Underscore, Ln, La, Lm, Le), "Humpty"),
        field!((Ll, La, Ls, Lt, Underscore, Ln, La, Lm, Le), "Drumpty"),
        field!((La, Lg, Le), 3),
    ];
    let u: NewUser = from_labelled_universalis(h);
    assert_eq!(
        u,
        NewUser {
            first_name: "Humpty",
            last_name: "Drumpty",
            age: 3,
        }
    );
}

#[test]
fn test_labelled_universalis_names() {
    type LastName = (Ll, La, Ls, Lt, Underscore, Ln, La, Lm, Le);
    type FirstName = (Lf, Li, Lr, Ls, Lt, Underscore, Ln, La, Lm, Le);

    let u = NewUser {
        first_name: "Humpty",
        last_name: "Drumpty",
        age: 3,
    };
    let h = into_labelled_universalis(u);
    let l_name_field: &Field<LastName, _> = h.get();
    assert_eq!(l_name_field.name, "last_name");
    let f_name_field: &Field<FirstName, _> = h.get();
    assert_eq!(f_name_field.name, "first_name");
}

#[test]
fn test_struct_into_labelled_universalis() {
    let u = NewUser {
        first_name: "Humpty",
        last_name: "Drumpty",
        age: 3,
    };
    let h = into_labelled_universalis(u);
    assert_eq!(
        h,
        hlist![
            field!(
                (Lf, Li, Lr, Ls, Lt, Underscore, Ln, La, Lm, Le),
                "Humpty",
                "first_name"
            ),
            field!(
                (Ll, La, Ls, Lt, Underscore, Ln, La, Lm, Le),
                "Drumpty",
                "last_name"
            ),
            field!((La, Lg, Le), 3, "age"),
        ]
    );
}

#[test]
fn test_reshaped_labelled_universalis_conversion() {
    let n_u = NormalUser {
        first_name: "Moe",
        last_name: "Ali",
        age: 30,
    };
    // Convert to labelled-Universalis representation
    let n_gen = into_labelled_universalis(n_u);
    // Reshape the labelled Universalis to fit the JumbledUser's Universalis Repr
    let (jumbled_gen, _): (<JumbledUser as NominataUniversalis>::Repr, _) = n_gen.sculpt();
    let j_u: JumbledUser = from_labelled_universalis(jumbled_gen); // Done

    assert_eq!(j_u.first_name, "Moe");
    assert_eq!(j_u.last_name, "Ali");
    assert_eq!(j_u.age, 30);
}

#[test]
fn test_aligned_nominata_convert_from() {
    let n_u = NormalUser {
        first_name: "Moe",
        last_name: "Ali",
        age: 30,
    };
    // even less boilerplate than before
    let j_u: JumbledUser = transform_from(n_u); // Done

    assert_eq!(j_u.first_name, "Moe");
    assert_eq!(j_u.last_name, "Ali");
    assert_eq!(j_u.age, 30);
}

#[test]
fn test_transfigure() {
    let internal_user = InternalUser {
        name: "John",
        age: 10,
        address: InternalAddress {
            is_whitelisted: true,
            name: "somewhere out there",
            phone: InternalPhoneNumber {
                main: 1234,
                secondary: None,
                emergency: Some(5678),
            },
        },
        is_banned: true,
        あ: true,
    };
    let expected_external_user = ExternalUser {
        name: "John",
        age: 10,
        address: ExternalAddress {
            name: "somewhere out there",
            phone: ExternalPhoneNumber { main: 1234 },
        },
        あ: true,
    };
    let external_user: ExternalUser = internal_user.transfigure();
    assert_eq!(external_user, expected_external_user);
}

type CreatedAt = (Lc, Lr, Le, La, Lt, Le, Ld, Underscore, La, Lt);

/// Converts from the Input type to the Output type,
/// provided that the Output type has a compatible labelled representation
/// with Input *AND* has a `created_at` Time field
///
/// If we wanted to, we could even make the time field type a parameter too for
/// even more generalisation.
///
/// Type parameters:
///
/// I stands for Input
/// O stands for Output
/// Indices is for the indices used for sculpting I with `created_at` Field into O's Universalis representation
fn to_audited<I, O, Indices>(o: I) -> O
where
    I: NominataUniversalis,
    O: NominataUniversalis,
    Coniunctio<Field<CreatedAt, OffsetDateTime>, <I as NominataUniversalis>::Repr>:
        Sculptor<<O as NominataUniversalis>::Repr, Indices>,
{
    // Add created_at field to NominataUniversalis repr of I
    let i_with_time = Coniunctio {
        head: field!(CreatedAt, OffsetDateTime::now_utc()),
        tail: into_labelled_universalis(o),
    };
    // sculpt it to fit Output NominataUniversalis representation
    let (compatible_with_o, _): (<O as NominataUniversalis>::Repr, _) = i_with_time.sculpt();
    // convert from NominataUniversalis to Output
    from_labelled_universalis(compatible_with_o)
}

#[test]
fn test_generalised_auditing() {
    let now = OffsetDateTime::now_utc().nanosecond();
    // Need to help the compiler out by annotating, but no biggie
    let n_u_audited: NormalUserWithAudit = to_audited(NormalUser::build());

    // We can even go from NormalUser to JumbledUser since they have compatible NominataUniversalis::Reprs
    let j_u_audited: JumbledUserWithAudit = to_audited(NormalUser::build());
    assert!(n_u_audited.created_at.nanosecond() >= now);
    assert!(j_u_audited.created_at.nanosecond() >= now);
}

#[test]
fn test_conversion_between_newtypes() {
    let s = "Foo".to_string();
    let nt = TypeWrapper(s.clone());
    let nt2: TypeWrapper2 = nt.transfigure();
    assert_eq!(nt2.0, s);
}

#[test]
fn test_transfigure_tuples() {
    let vec4 = Vec4f(1.0, 2.0, 0.0, 3.0);
    let vec3 = vec4.transfigure();
    assert_eq!(Vec3f(1.0, 2.0, 0.0), vec3);
}

#[test]
fn test_enum_from_labelled_universalis() {
    let variant_a = Disiunctio::inject(field!((Uv, La, Lr, Li, La, Ln, Lt, Ua), hlist![]));
    let variant_b = Disiunctio::inject(field!(
        (Uv, La, Lr, Li, La, Ln, Lt, Ub),
        hlist![field!((Underscore, N0), 42i32)]
    ));
    let variant_c = Disiunctio::inject(field!(
        (Uv, La, Lr, Li, La, Ln, Lt, Uc),
        hlist![field!(Lx, "test".into()), field!(Ly, true)]
    ));
    assert_eq!(
        from_labelled_universalis::<LabelledEnum1, _>(variant_a),
        LabelledEnum1::VariantA,
    );
    assert_eq!(
        from_labelled_universalis::<LabelledEnum1, _>(variant_b),
        LabelledEnum1::VariantB(42),
    );
    assert_eq!(
        from_labelled_universalis::<LabelledEnum1, _>(variant_c),
        LabelledEnum1::VariantC {
            x: "test".into(),
            y: true
        },
    );
}

#[test]
fn test_enum_into_labelled_universalis() {
    let variant_a = into_labelled_universalis(LabelledEnum1::VariantA);
    let variant_b = into_labelled_universalis(LabelledEnum1::VariantB(42));
    let variant_c = into_labelled_universalis(LabelledEnum1::VariantC {
        x: "test".into(),
        y: true,
    });
    assert_eq!(
        variant_a,
        Disiunctio::inject(field!(
            (Uv, La, Lr, Li, La, Ln, Lt, Ua),
            hlist![],
            "VariantA"
        )),
    );
    assert_eq!(
        variant_b,
        Disiunctio::inject(field!(
            (Uv, La, Lr, Li, La, Ln, Lt, Ub),
            hlist![field!((Underscore, N0), 42i32, "_0")],
            "VariantB"
        )),
    );
    assert_eq!(
        variant_c,
        Disiunctio::inject(field!(
            (Uv, La, Lr, Li, La, Ln, Lt, Uc),
            hlist![field!(Lx, "test".into(), "x"), field!(Ly, true, "y")],
            "VariantC"
        ))
    );
}

#[test]
fn test_sculpt_enum() {
    let value = LabelledEnum1::VariantC {
        x: "test".into(),
        y: true,
    };
    let repr = match into_labelled_universalis(value).subset() {
        Ok(repr) => repr,
        Err(rem) => match rem {}, // should be unreachable
    };
    let new_value: LabelledEnum2 = from_labelled_universalis(repr);

    assert_eq!(
        new_value,
        LabelledEnum2::VariantC {
            x: "test".into(),
            y: true
        }
    );
}

#[test]
fn test_transfigure_keyword_field_structs() {
    let value = HasKeyword1 { r#type: 3 };
    let result: HasKeyword2 = value.transfigure();
    assert_eq!(3, result.r#type);
}

#[test]
fn test_transfigure_keyword_field_embedder_structs() {
    let value = {
        let embedded = HasKeyword1 { r#type: 3 };
        HasKeyword1Embedder { r#true: embedded }
    };
    let result: HasKeyword2Embedder = value.transfigure();
    assert_eq!(3, result.r#true.r#type);
}
