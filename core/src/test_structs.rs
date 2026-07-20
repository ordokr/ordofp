//! Defines a bunch of unit structs with unique types for use by unit tests in ordofp.
//!
//! The theme is "I need a bunch of types and I need them NOW!"

pub mod unit_copy {
    macro_rules! make_unit_struct {
        ($($Name:ident)*) => {$(
            /// Unit-like struct that implements Copy, for tests.
            #[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
            pub struct $Name;
        )*};
    }
    // Feel free to add more as necessary
    make_unit_struct! {A B C D E _F}
}

pub mod unit_move {
    macro_rules! make_unit_struct {
        ($($Name:ident)*) => {$(
            /// Unit-like struct that does not implement Copy, for tests.
            #[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
            pub struct $Name;
        )*};
    }
    // Feel free to add more as necessary
    make_unit_struct! {_A _B _C _D _E _F}
}

pub mod i32_copy {
    macro_rules! make_unit_struct {
        ($($Name:ident)*) => {$(
            /// Newtype around i32 that implements Copy, for tests.
            #[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
            pub struct $Name(pub i32);
        )*};
    }
    // Feel free to add more as necessary
    make_unit_struct! {_X _Y _Z}
}

pub mod i32_move {
    macro_rules! make_unit_struct {
        ($($Name:ident)*) => {$(
            /// Newtype around i32 that does not implement Copy, for tests.
            #[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
            pub struct $Name(pub i32);
        )*};
    }
    // Feel free to add more as necessary
    make_unit_struct! {_X _Y _Z}
}
