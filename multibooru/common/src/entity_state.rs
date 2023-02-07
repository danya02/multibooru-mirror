/// This macro takes a list of associations such as:
/// ```
/// # use serde::{Serialize, Deserialize};
/// # #[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
/// # pub struct PostState;
/// # #[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
/// # pub struct TagState;
/// # #[macro_use] extern crate common;
/// # fn main() {}
/// make_entity_state! {
///  Thing, ThingState {
///    Post: u64 => PostState,
///    Tag: String => TagState,
///   }
/// }
/// ```
///
/// and generates two enums that look like:
/// ```
/// # struct PostState;
/// # struct TagState;
/// enum Thing {
///   Post(u64),
///   Tag(String),
/// }
/// enum ThingState {
///   Post(u64, PostState),
///   Tag(String, TagState),
/// }
///
/// impl From<ThingState> for Thing {
///   fn from(state: ThingState) -> Self {
///     match state {
///       ThingState::Post(id, _) => Thing::Post(id),
///       ThingState::Tag(id, _) => Thing::Tag(id),
///     }
///   }
/// }
/// ```
#[macro_export]
macro_rules! make_entity_state {
    ($name:ident, $state_name:ident { $($assoc:ident: $id:ty => $state:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(tag = "entity", content = "id")]
        pub enum $name {
            $(
                $assoc($id),
            )*
        }

        #[derive(Debug, Clone, PartialEq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(tag = "entity", content = "data")]
        pub enum $state_name {
            $(
                $assoc{
                    id: $id,
                    state: $state
                },
            )*
        }

        impl From<$state_name> for $name {
            fn from(state: $state_name) -> Self {
                state.get_entity()
            }
        }

        impl $state_name {
            pub fn get_entity(&self) -> $name {
                match self {
                    $(
                        $state_name::$assoc{ id, .. } => $name::$assoc((*id).clone()),
                    )*
                }
            }
        }
    };
}
