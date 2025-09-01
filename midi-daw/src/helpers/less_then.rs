use bevy::prelude::*;

// TODO: Make a macro to build a "LessThan" type for any given numeric type

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct UsizeLessThan<const LT: usize>(usize);

impl<const LT: usize> TryFrom<usize> for UsizeLessThan<LT> {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < LT {
            Ok(Self(value))
        } else {
            Err(format!(
                "Too big Error! {value} was expected to be less than {LT}"
            ))
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut)]
// pub struct UsizeLessThan<DT, const NUM: DT>(pub DT)
// where
//     DT: QuacksLikeANumber;
// {
//     // less_then: DT,
//     // #[deref]
//     value: DT,
// }

// impl<const LT: usize> TryFrom<usize> for UsizeLessThan<LT> {
//     type Error = String;
//
//     fn try_from(value: usize) -> Result<Self, Self::Error> {
//         if value < LT {
//             Ok(Self(value))
//         } else {
//             Err(format!(
//                 "Too big Error! {value} was expected to be less than {LT}"
//             ))
//         }
//     }
// }
