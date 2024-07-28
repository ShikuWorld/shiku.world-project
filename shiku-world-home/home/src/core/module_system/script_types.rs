#![allow(clippy::all)]
use rhai::plugin::*;

use rhai::{CustomType, TypeBuilder};

#[derive(Clone, CustomType)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[export_module]
pub mod CharacterDirectionModule {
    use crate::core::blueprint::character_animation::CharacterDirection;
    pub const Left: CharacterDirection = CharacterDirection::Left;
    pub const Down: CharacterDirection = CharacterDirection::Down;
    pub const Up: CharacterDirection = CharacterDirection::Up;
    pub const Right: CharacterDirection = CharacterDirection::Right;

    // Printing
    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(my_enum: &mut CharacterDirection) -> String {
        format!("{my_enum:?}")
    }

    // '==' and '!=' operators
    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(my_enum: &mut CharacterDirection, my_enum2: CharacterDirection) -> bool {
        my_enum == &my_enum2
    }
    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(my_enum: &mut CharacterDirection, my_enum2: CharacterDirection) -> bool {
        my_enum != &my_enum2
    }
}
