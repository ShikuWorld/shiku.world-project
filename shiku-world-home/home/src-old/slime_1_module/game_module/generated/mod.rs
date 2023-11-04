#![allow(dead_code)]
#![allow(unused_variables)]

use crate::core::entity_manager::{ColliderEntityMap, EntityManager};
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::terrain_gen::condense_terrain_from_tiles;
use crate::core::tween::Tween;
use crate::resource_module::def::GuestId;
use crate::resource_module::map::def::{
    GeneralObject, LayerName, ObjectId, TerrainChunk, TiledMap,
};
use log::debug;

use crate::core::entity::def::{Entity, EntityId, RemoveEntity, ShowEntity, UpdateEntity};
use crate::core::entity::physics::{
    physical_shape_from_general_obj, Physical, PhysicalShape, PhysicsArea, PhysicsNone,
    PhysicsRigidBody, PhysicsStaticRigidBody, ShapeRect,
};
use crate::core::entity::render::{
    NoRender, RenderTypeText, RenderTypeTimer, ShowEffect, StaticImage,
};
use crate::core::entity::terrain::TerrainEntity;

use rapier2d::prelude::{ColliderHandle, Isometry, Real, Vector};

use serde::Deserialize;
use snowflake::SnowflakeIdBucket;
use std::collections::HashMap;
use std::iter::{Filter, Map};

#[derive(Debug, Deserialize, PartialEq)]
pub enum Slime1GameObject {
    Terrain,
    BGDecoration,
    CaveHeartWall,
    DeathArea,
    DeathProp,
    DeathReviveArea,
    DeathReviveStatue,
    DeathTorch,
    Debris,
    Door,
    EnterArea,
    ExitArea,
    GreatWaterfallTiles,
    Guest,
    GuestDead,
    GuestNameplate,
    Heart,
    HeartSmoll,
    HeartWallGlow,
    Observer,
    OpeningAreaPlate,
    Secret,
    SlimeCharge,
    SlimeLight,
    TeleportEnd,
    TeleportStart,
    Timer,
    WallPlatform,
    WallPlatformOpener,
}

#[derive(Debug)]
pub struct BGDecoration;

#[derive(Debug)]
pub struct CaveHeartWall {
    pub blue_heart: EntityId,
    pub cave_open: bool,
    pub door_to_remove: EntityId,
    pub red_heart: EntityId,
    pub yellow_heart: EntityId,
}

#[derive(Debug)]
pub struct DeathArea {
    pub torch: EntityId,
}

#[derive(Debug)]
pub struct DeathProp;

#[derive(Debug)]
pub struct DeathReviveArea;

#[derive(Debug)]
pub struct DeathReviveStatue {
    pub revive_area: EntityId,
}

#[derive(Debug)]
pub struct DeathTorch;

#[derive(Debug)]
pub struct Debris;

#[derive(Debug)]
pub struct Door {
    pub is_open: bool,
    pub open_change_position: Tween,
}

#[derive(Debug)]
pub struct EnterArea {
    pub slot_id: String,
}

#[derive(Debug)]
pub struct ExitArea {
    pub slot_id: String,
}

#[derive(Debug)]
pub struct GreatWaterfallTiles;

#[derive(Debug)]
pub struct Guest {
    pub heart_color: String,
}

#[derive(Debug)]
pub struct GuestDead {
    pub flame: EntityId,
    pub heart_color: String,
    pub slime: EntityId,
}

#[derive(Debug)]
pub struct GuestNameplate;

#[derive(Debug)]
pub struct Heart {
    pub color: String,
}

#[derive(Debug)]
pub struct HeartSmoll {
    pub color: String,
}

#[derive(Debug)]
pub struct HeartWallGlow {
    pub color: String,
}

#[derive(Debug)]
pub struct Observer;

#[derive(Debug)]
pub struct OpeningAreaPlate {
    pub door_to_open_1: EntityId,
    pub door_to_open_2: EntityId,
    pub door_to_open_3: EntityId,
    pub door_to_open_4: EntityId,
    pub opener_variant: String,
}

#[derive(Debug)]
pub struct Secret {
    pub secret_name: String,
}

#[derive(Debug)]
pub struct SlimeCharge;

#[derive(Debug)]
pub struct SlimeLight;

#[derive(Debug)]
pub struct TeleportEnd;

#[derive(Debug)]
pub struct TeleportStart {
    pub teleport_end: EntityId,
}

#[derive(Debug)]
pub struct Timer;

#[derive(Debug)]
pub struct WallPlatform {
    pub off: bool,
    pub opener: EntityId,
    pub opener_2: EntityId,
    pub opener_3: EntityId,
    pub wall_variant: String,
}

#[derive(Debug)]
pub struct WallPlatformOpener {
    pub active: bool,
    pub heart_color: String,
    pub opener_variant: String,
}

pub struct CaveHeartWallVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl CaveHeartWallVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct CaveHeartWallVariants {
    pub default: CaveHeartWallVariant,
}

impl CaveHeartWall {
    pub const VARIANTS: CaveHeartWallVariants = CaveHeartWallVariants {
        default: CaveHeartWallVariant {
            gid_default: "547",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 162.0,
                height: 72.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl CaveHeartWallVariants {
    pub fn get_variant(&self, variant: &String) -> &CaveHeartWallVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct DeathReviveStatueVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl DeathReviveStatueVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct DeathReviveStatueVariants {
    pub default: DeathReviveStatueVariant,
}

impl DeathReviveStatue {
    pub const VARIANTS: DeathReviveStatueVariants = DeathReviveStatueVariants {
        default: DeathReviveStatueVariant {
            gid_default: "915",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.5,
                offset_from_center_y: 15.0,
                width: 25.0,
                height: 34.0,
            }),
            offset_from_center_x: 0.50,
            offset_from_center_y: 15.00,
        },
    };
}

impl DeathReviveStatueVariants {
    pub fn get_variant(&self, variant: &String) -> &DeathReviveStatueVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct DeathTorchVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl DeathTorchVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct DeathTorchVariants {
    pub default: DeathTorchVariant,
}

impl DeathTorch {
    pub const VARIANTS: DeathTorchVariants = DeathTorchVariants {
        default: DeathTorchVariant {
            gid_default: "905",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 8.0,
                width: 20.0,
                height: 16.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 8.00,
        },
    };
}

impl DeathTorchVariants {
    pub fn get_variant(&self, variant: &String) -> &DeathTorchVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct DebrisVariant {
    pub gid_formation_3: &'static str,
    pub gid_stone_l_4: &'static str,
    pub gid_stone_l_2: &'static str,
    pub gid_formation_5: &'static str,
    pub gid_stone_l_3: &'static str,
    pub gid_stone_m_3: &'static str,
    pub gid_stone_s_4: &'static str,
    pub gid_stone_s_3: &'static str,
    pub gid_stone_s_1: &'static str,
    pub gid_stone_l_1: &'static str,
    pub gid_formation_2: &'static str,
    pub gid_formation_1: &'static str,
    pub gid_stone_m_6: &'static str,
    pub gid_stone_m_5: &'static str,
    pub gid_stone_m_2: &'static str,
    pub gid_stone_s_2: &'static str,
    pub gid_stone_s_5: &'static str,
    pub gid_formation_4: &'static str,
    pub gid_stone_m_1: &'static str,
    pub gid_default: &'static str,
    pub gid_stone_m_4: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl DebrisVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct DebrisVariants {
    pub default: DebrisVariant,
}

impl Debris {
    pub const VARIANTS: DebrisVariants = DebrisVariants {
        default: DebrisVariant {
            gid_formation_3: "592",
            gid_stone_l_4: "609",
            gid_stone_l_2: "607",
            gid_formation_5: "594",
            gid_stone_l_3: "608",
            gid_stone_m_3: "595",
            gid_stone_s_4: "598",
            gid_stone_s_3: "599",
            gid_stone_s_1: "601",
            gid_stone_l_1: "606",
            gid_formation_2: "591",
            gid_formation_1: "590",
            gid_stone_m_6: "605",
            gid_stone_m_5: "604",
            gid_stone_m_2: "596",
            gid_stone_s_2: "600",
            gid_stone_s_5: "602",
            gid_formation_4: "593",
            gid_stone_m_1: "597",
            gid_default: "589",
            gid_stone_m_4: "603",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl DebrisVariants {
    pub fn get_variant(&self, variant: &String) -> &DebrisVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct DoorVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl DoorVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct DoorVariants {
    pub plattform_l: DoorVariant,
    pub golden: DoorVariant,
    pub plattform_xl: DoorVariant,
    pub tree_left: DoorVariant,
    pub forest_s: DoorVariant,
    pub forest_l: DoorVariant,
    pub plattform_s: DoorVariant,
    pub default: DoorVariant,
    pub forest_m: DoorVariant,
    pub plattform_m: DoorVariant,
    pub tree_right: DoorVariant,
}

impl Door {
    pub const VARIANTS: DoorVariants = DoorVariants {
        plattform_l: DoorVariant {
            gid_default: "104",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -0.5,
                offset_from_center_y: 0.5,
                width: 79.0,
                height: 17.0,
            }),
            offset_from_center_x: -0.50,
            offset_from_center_y: 0.50,
        },
        golden: DoorVariant {
            gid_default: "102",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 24.0,
                height: 64.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        plattform_xl: DoorVariant {
            gid_default: "103",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -0.5,
                offset_from_center_y: 0.5,
                width: 113.0,
                height: 17.0,
            }),
            offset_from_center_x: -0.50,
            offset_from_center_y: 0.50,
        },
        tree_left: DoorVariant {
            gid_default: "110",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -16.0,
                offset_from_center_y: -5.0,
                width: 32.0,
                height: 6.0,
            }),
            offset_from_center_x: -16.00,
            offset_from_center_y: -5.00,
        },
        forest_s: DoorVariant {
            gid_default: "107",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -1.0,
                width: 16.0,
                height: 6.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -1.00,
        },
        forest_l: DoorVariant {
            gid_default: "109",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -1.5,
                width: 48.0,
                height: 6.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -1.50,
        },
        plattform_s: DoorVariant {
            gid_default: "106",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -0.5,
                offset_from_center_y: 0.5,
                width: 31.0,
                height: 17.0,
            }),
            offset_from_center_x: -0.50,
            offset_from_center_y: 0.50,
        },
        default: DoorVariant {
            gid_default: "101",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 24.0,
                height: 64.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        forest_m: DoorVariant {
            gid_default: "108",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -1.0,
                width: 32.0,
                height: 6.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -1.00,
        },
        plattform_m: DoorVariant {
            gid_default: "105",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.5,
                width: 46.0,
                height: 17.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.50,
        },
        tree_right: DoorVariant {
            gid_default: "111",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 16.0,
                offset_from_center_y: -5.0,
                width: 32.0,
                height: 6.0,
            }),
            offset_from_center_x: 16.00,
            offset_from_center_y: -5.00,
        },
    };
}

impl DoorVariants {
    pub fn get_variant(&self, variant: &String) -> &DoorVariant {
        match variant.as_str() {
            "plattform_l" => &self.plattform_l,
            "golden" => &self.golden,
            "plattform_xl" => &self.plattform_xl,
            "tree_left" => &self.tree_left,
            "forest_s" => &self.forest_s,
            "forest_l" => &self.forest_l,
            "plattform_s" => &self.plattform_s,
            "default" => &self.default,
            "forest_m" => &self.forest_m,
            "plattform_m" => &self.plattform_m,
            "tree_right" => &self.tree_right,
            _ => &self.default,
        }
    }
}

pub struct GuestVariant {
    pub gid_idle_2: &'static str,
    pub gid_jump_hold_left_3: &'static str,
    pub gid_jump_hold_up_2: &'static str,
    pub gid_face_right: &'static str,
    pub gid_jump_hold_right_1: &'static str,
    pub gid_jumping_up: &'static str,
    pub gid_extend_right_2: &'static str,
    pub gid_jump_hold_right_2: &'static str,
    pub gid_jump_hold_right_3: &'static str,
    pub gid_jump_hold_up_3: &'static str,
    pub gid_extend_left_1: &'static str,
    pub gid_idle_3: &'static str,
    pub gid_moved_left: &'static str,
    pub gid_jump_hold_up_1: &'static str,
    pub gid_jumping_right: &'static str,
    pub gid_extend_left_2: &'static str,
    pub gid_moved_right: &'static str,
    pub gid_jump_hold_left_1: &'static str,
    pub gid_face_left: &'static str,
    pub gid_idle_1: &'static str,
    pub gid_idle_4: &'static str,
    pub gid_extend_right_1: &'static str,
    pub gid_jump_hold_left_2: &'static str,
    pub gid_jumping_left: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl GuestVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct GuestVariants {
    pub blue: GuestVariant,
    pub albino: GuestVariant,
    pub red: GuestVariant,
    pub orange: GuestVariant,
    pub purple: GuestVariant,
    pub default: GuestVariant,
    pub black: GuestVariant,
    pub pink: GuestVariant,
}

impl Guest {
    pub const VARIANTS: GuestVariants = GuestVariants {
        blue: GuestVariant {
            gid_moved_right: "143",
            gid_face_left: "148",
            gid_extend_right_2: "142",
            gid_jump_hold_up_1: "144",
            gid_jumping_right: "155",
            gid_idle_4: "139",
            gid_extend_right_1: "141",
            gid_extend_left_1: "149",
            gid_jump_hold_left_3: "158",
            gid_jump_hold_right_2: "153",
            gid_extend_left_2: "150",
            gid_jump_hold_left_1: "156",
            gid_jumping_up: "147",
            gid_jump_hold_up_3: "146",
            gid_jump_hold_left_2: "157",
            gid_idle_2: "137",
            gid_idle_3: "138",
            gid_moved_left: "151",
            gid_idle_1: "136",
            gid_jumping_left: "159",
            gid_jump_hold_right_3: "154",
            gid_jump_hold_right_1: "152",
            gid_jump_hold_up_2: "145",
            gid_face_right: "140",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -0.5,
                width: 12.0,
                height: 9.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -0.50,
        },
        albino: GuestVariant {
            gid_extend_left_1: "245",
            gid_jump_hold_up_2: "241",
            gid_jump_hold_right_1: "248",
            gid_jump_hold_right_3: "250",
            gid_extend_left_2: "246",
            gid_jump_hold_up_1: "240",
            gid_moved_right: "239",
            gid_face_left: "244",
            gid_moved_left: "247",
            gid_extend_right_2: "238",
            gid_jump_hold_up_3: "242",
            gid_jump_hold_left_3: "254",
            gid_idle_2: "233",
            gid_jumping_right: "251",
            gid_idle_4: "235",
            gid_jumping_up: "243",
            gid_jump_hold_right_2: "249",
            gid_idle_3: "234",
            gid_extend_right_1: "237",
            gid_jumping_left: "255",
            gid_jump_hold_left_2: "253",
            gid_jump_hold_left_1: "252",
            gid_face_right: "236",
            gid_idle_1: "232",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        red: GuestVariant {
            gid_idle_2: "161",
            gid_idle_3: "162",
            gid_extend_left_2: "174",
            gid_extend_right_1: "165",
            gid_idle_4: "163",
            gid_face_right: "164",
            gid_extend_right_2: "166",
            gid_jump_hold_right_1: "176",
            gid_moved_left: "175",
            gid_jump_hold_up_3: "170",
            gid_jumping_right: "179",
            gid_jump_hold_right_3: "178",
            gid_jump_hold_up_1: "168",
            gid_jump_hold_right_2: "177",
            gid_extend_left_1: "173",
            gid_idle_1: "160",
            gid_face_left: "172",
            gid_moved_right: "167",
            gid_jumping_up: "171",
            gid_jump_hold_up_2: "169",
            gid_jump_hold_left_2: "181",
            gid_jump_hold_left_1: "180",
            gid_jumping_left: "183",
            gid_jump_hold_left_3: "182",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        orange: GuestVariant {
            gid_face_right: "212",
            gid_idle_3: "210",
            gid_jump_hold_up_1: "216",
            gid_extend_right_1: "213",
            gid_extend_left_1: "221",
            gid_moved_left: "223",
            gid_jumping_up: "219",
            gid_moved_right: "215",
            gid_jump_hold_right_2: "225",
            gid_extend_right_2: "214",
            gid_jumping_right: "227",
            gid_jump_hold_left_1: "228",
            gid_jump_hold_up_3: "218",
            gid_extend_left_2: "222",
            gid_jump_hold_right_3: "226",
            gid_jump_hold_left_3: "230",
            gid_idle_1: "208",
            gid_jump_hold_left_2: "229",
            gid_jumping_left: "231",
            gid_jump_hold_up_2: "217",
            gid_idle_4: "211",
            gid_idle_2: "209",
            gid_jump_hold_right_1: "224",
            gid_face_left: "220",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        purple: GuestVariant {
            gid_jump_hold_up_2: "193",
            gid_jump_hold_right_2: "201",
            gid_jump_hold_up_3: "194",
            gid_jump_hold_left_3: "206",
            gid_idle_2: "185",
            gid_jumping_up: "195",
            gid_face_right: "188",
            gid_extend_left_2: "198",
            gid_extend_left_1: "197",
            gid_face_left: "196",
            gid_extend_right_1: "189",
            gid_jump_hold_right_1: "200",
            gid_moved_left: "199",
            gid_extend_right_2: "190",
            gid_idle_1: "184",
            gid_jump_hold_right_3: "202",
            gid_jumping_right: "203",
            gid_idle_4: "187",
            gid_jump_hold_left_2: "205",
            gid_idle_3: "186",
            gid_jump_hold_left_1: "204",
            gid_jumping_left: "207",
            gid_moved_right: "191",
            gid_jump_hold_up_1: "192",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        default: GuestVariant {
            gid_idle_2: "113",
            gid_jump_hold_left_3: "134",
            gid_jump_hold_up_2: "121",
            gid_face_right: "116",
            gid_jump_hold_right_1: "128",
            gid_jumping_up: "123",
            gid_extend_right_2: "118",
            gid_jump_hold_right_2: "129",
            gid_jump_hold_right_3: "130",
            gid_jump_hold_up_3: "122",
            gid_extend_left_1: "125",
            gid_idle_3: "114",
            gid_moved_left: "127",
            gid_jump_hold_up_1: "120",
            gid_jumping_right: "131",
            gid_extend_left_2: "126",
            gid_moved_right: "119",
            gid_jump_hold_left_1: "132",
            gid_face_left: "124",
            gid_idle_1: "112",
            gid_idle_4: "115",
            gid_extend_right_1: "117",
            gid_jump_hold_left_2: "133",
            gid_jumping_left: "135",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -0.5,
                width: 12.0,
                height: 9.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -0.50,
        },
        black: GuestVariant {
            gid_jumping_up: "267",
            gid_idle_1: "256",
            gid_jump_hold_right_2: "273",
            gid_jump_hold_up_2: "265",
            gid_moved_right: "263",
            gid_idle_4: "259",
            gid_idle_3: "258",
            gid_jumping_right: "275",
            gid_jump_hold_up_1: "264",
            gid_jump_hold_left_1: "276",
            gid_moved_left: "271",
            gid_jumping_left: "279",
            gid_jump_hold_right_1: "272",
            gid_extend_right_2: "262",
            gid_face_left: "268",
            gid_jump_hold_right_3: "274",
            gid_extend_right_1: "261",
            gid_idle_2: "257",
            gid_extend_left_2: "270",
            gid_jump_hold_up_3: "266",
            gid_jump_hold_left_3: "278",
            gid_jump_hold_left_2: "277",
            gid_face_right: "260",
            gid_extend_left_1: "269",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        pink: GuestVariant {
            gid_idle_3: "282",
            gid_jump_hold_right_1: "296",
            gid_jump_hold_right_2: "297",
            gid_extend_left_2: "294",
            gid_jump_hold_right_3: "298",
            gid_extend_left_1: "293",
            gid_jump_hold_left_2: "301",
            gid_jumping_left: "303",
            gid_jump_hold_up_3: "290",
            gid_jump_hold_left_1: "300",
            gid_idle_1: "280",
            gid_extend_right_1: "285",
            gid_moved_right: "287",
            gid_jump_hold_left_3: "302",
            gid_face_right: "284",
            gid_face_left: "292",
            gid_jump_hold_up_2: "289",
            gid_jump_hold_up_1: "288",
            gid_jumping_right: "299",
            gid_idle_4: "283",
            gid_moved_left: "295",
            gid_extend_right_2: "286",
            gid_idle_2: "281",
            gid_jumping_up: "291",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl GuestVariants {
    pub fn get_variant(&self, variant: &String) -> &GuestVariant {
        match variant.as_str() {
            "blue" => &self.blue,
            "albino" => &self.albino,
            "red" => &self.red,
            "orange" => &self.orange,
            "purple" => &self.purple,
            "default" => &self.default,
            "black" => &self.black,
            "pink" => &self.pink,
            _ => &self.default,
        }
    }
}

pub struct GuestDeadVariant {
    pub gid_right: &'static str,
    pub gid_left: &'static str,
    pub gid_idle: &'static str,
    pub gid_up: &'static str,
    pub gid_down: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl GuestDeadVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct GuestDeadVariants {
    pub black: GuestDeadVariant,
    pub albino: GuestDeadVariant,
    pub blue: GuestDeadVariant,
    pub purple: GuestDeadVariant,
    pub red: GuestDeadVariant,
    pub orange: GuestDeadVariant,
    pub pink: GuestDeadVariant,
    pub default: GuestDeadVariant,
}

impl GuestDead {
    pub const VARIANTS: GuestDeadVariants = GuestDeadVariants {
        black: GuestDeadVariant {
            gid_right: "950",
            gid_idle: "948",
            gid_down: "947",
            gid_up: "946",
            gid_left: "949",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        albino: GuestDeadVariant {
            gid_idle: "938",
            gid_right: "940",
            gid_left: "939",
            gid_up: "936",
            gid_down: "937",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        blue: GuestDeadVariant {
            gid_right: "925",
            gid_down: "922",
            gid_idle: "923",
            gid_up: "921",
            gid_left: "924",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        purple: GuestDeadVariant {
            gid_left: "929",
            gid_down: "927",
            gid_right: "930",
            gid_idle: "928",
            gid_up: "926",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        red: GuestDeadVariant {
            gid_left: "944",
            gid_down: "942",
            gid_idle: "943",
            gid_up: "941",
            gid_right: "945",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        orange: GuestDeadVariant {
            gid_left: "954",
            gid_down: "952",
            gid_up: "951",
            gid_idle: "953",
            gid_right: "955",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        pink: GuestDeadVariant {
            gid_up: "931",
            gid_down: "932",
            gid_idle: "933",
            gid_left: "934",
            gid_right: "935",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
        default: GuestDeadVariant {
            gid_right: "920",
            gid_left: "919",
            gid_idle: "918",
            gid_up: "916",
            gid_down: "917",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: -2.0,
                width: 10.0,
                height: 8.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: -2.00,
        },
    };
}

impl GuestDeadVariants {
    pub fn get_variant(&self, variant: &String) -> &GuestDeadVariant {
        match variant.as_str() {
            "black" => &self.black,
            "albino" => &self.albino,
            "blue" => &self.blue,
            "purple" => &self.purple,
            "red" => &self.red,
            "orange" => &self.orange,
            "pink" => &self.pink,
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct HeartVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl HeartVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct HeartVariants {
    pub default: HeartVariant,
    pub yellow: HeartVariant,
    pub blue: HeartVariant,
}

impl Heart {
    pub const VARIANTS: HeartVariants = HeartVariants {
        default: HeartVariant {
            gid_default: "419",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -1.0,
                offset_from_center_y: 0.5,
                width: 12.0,
                height: 14.0,
            }),
            offset_from_center_x: -1.00,
            offset_from_center_y: 0.50,
        },
        yellow: HeartVariant {
            gid_default: "427",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -1.0,
                offset_from_center_y: -0.5,
                width: 12.0,
                height: 14.0,
            }),
            offset_from_center_x: -1.00,
            offset_from_center_y: -0.50,
        },
        blue: HeartVariant {
            gid_default: "435",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -1.0,
                offset_from_center_y: -1.5,
                width: 12.0,
                height: 14.0,
            }),
            offset_from_center_x: -1.00,
            offset_from_center_y: -1.50,
        },
    };
}

impl HeartVariants {
    pub fn get_variant(&self, variant: &String) -> &HeartVariant {
        match variant.as_str() {
            "default" => &self.default,
            "yellow" => &self.yellow,
            "blue" => &self.blue,
            _ => &self.default,
        }
    }
}

pub struct HeartSmollVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl HeartSmollVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct HeartSmollVariants {
    pub yellow: HeartSmollVariant,
    pub default: HeartSmollVariant,
    pub blue: HeartSmollVariant,
}

impl HeartSmoll {
    pub const VARIANTS: HeartSmollVariants = HeartSmollVariants {
        yellow: HeartSmollVariant {
            gid_default: "483",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        default: HeartSmollVariant {
            gid_default: "443",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        blue: HeartSmollVariant {
            gid_default: "463",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl HeartSmollVariants {
    pub fn get_variant(&self, variant: &String) -> &HeartSmollVariant {
        match variant.as_str() {
            "yellow" => &self.yellow,
            "default" => &self.default,
            "blue" => &self.blue,
            _ => &self.default,
        }
    }
}

pub struct HeartWallGlowVariant {
    pub gid_glow: &'static str,
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl HeartWallGlowVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct HeartWallGlowVariants {
    pub yellow: HeartWallGlowVariant,
    pub default: HeartWallGlowVariant,
    pub red: HeartWallGlowVariant,
}

impl HeartWallGlow {
    pub const VARIANTS: HeartWallGlowVariants = HeartWallGlowVariants {
        yellow: HeartWallGlowVariant {
            gid_default: "580",
            gid_glow: "570",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        default: HeartWallGlowVariant {
            gid_glow: "548",
            gid_default: "558",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        red: HeartWallGlowVariant {
            gid_glow: "559",
            gid_default: "569",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl HeartWallGlowVariants {
    pub fn get_variant(&self, variant: &String) -> &HeartWallGlowVariant {
        match variant.as_str() {
            "yellow" => &self.yellow,
            "default" => &self.default,
            "red" => &self.red,
            _ => &self.default,
        }
    }
}

pub struct ObserverVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl ObserverVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct ObserverVariants {
    pub default: ObserverVariant,
}

impl Observer {
    pub const VARIANTS: ObserverVariants = ObserverVariants {
        default: ObserverVariant {
            gid_default: "353",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 16.0,
                height: 16.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl ObserverVariants {
    pub fn get_variant(&self, variant: &String) -> &ObserverVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct OpeningAreaPlateVariant {
    pub gid_activated: &'static str,
    pub gid_done: &'static str,
    pub gid_default: &'static str,
    pub shape_platform: PhysicalShape,
    pub shape_activation: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl OpeningAreaPlateVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct OpeningAreaPlateVariants {
    pub portal_green_no_glow: OpeningAreaPlateVariant,
    pub portal: OpeningAreaPlateVariant,
    pub green_opener: OpeningAreaPlateVariant,
    pub default: OpeningAreaPlateVariant,
    pub portal_no_glow: OpeningAreaPlateVariant,
    pub portal_green: OpeningAreaPlateVariant,
}

impl OpeningAreaPlate {
    pub const VARIANTS: OpeningAreaPlateVariants = OpeningAreaPlateVariants {
        portal_green_no_glow: OpeningAreaPlateVariant {
            gid_activated: "340",
            gid_done: "345",
            gid_default: "339",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 11.0,
                width: 24.0,
                height: 6.0,
            }),
            offset_from_center_x: 4.00,
            offset_from_center_y: 11.00,
            shape_activation: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 2.0,
                width: 14.0,
                height: 12.0,
            }),
        },
        portal: OpeningAreaPlateVariant {
            gid_activated: "319",
            gid_default: "318",
            gid_done: "324",
            shape_activation: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 2.0,
                width: 14.0,
                height: 12.0,
            }),
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 11.0,
                width: 24.0,
                height: 6.0,
            }),
            offset_from_center_x: 4.00,
            offset_from_center_y: 11.00,
        },
        green_opener: OpeningAreaPlateVariant {
            gid_done: "352",
            gid_default: "346",
            gid_activated: "347",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 11.0,
                width: 24.0,
                height: 6.0,
            }),
            offset_from_center_x: 4.00,
            offset_from_center_y: 11.00,
            shape_activation: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 2.0,
                width: 14.0,
                height: 12.0,
            }),
        },
        default: OpeningAreaPlateVariant {
            gid_activated: "312",
            gid_done: "317",
            gid_default: "311",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 11.0,
                width: 24.0,
                height: 6.0,
            }),
            offset_from_center_x: 4.00,
            offset_from_center_y: 11.00,
            shape_activation: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 2.0,
                width: 14.0,
                height: 12.0,
            }),
        },
        portal_no_glow: OpeningAreaPlateVariant {
            gid_default: "325",
            gid_done: "331",
            gid_activated: "326",
            shape_activation: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 2.0,
                width: 14.0,
                height: 12.0,
            }),
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 3.75,
                offset_from_center_y: 11.125,
                width: 24.0,
                height: 6.0,
            }),
            offset_from_center_x: 3.75,
            offset_from_center_y: 11.12,
        },
        portal_green: OpeningAreaPlateVariant {
            gid_done: "338",
            gid_default: "332",
            gid_activated: "333",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 11.0,
                width: 24.0,
                height: 6.0,
            }),
            offset_from_center_x: 4.00,
            offset_from_center_y: 11.00,
            shape_activation: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 4.0,
                offset_from_center_y: 2.0,
                width: 14.0,
                height: 12.0,
            }),
        },
    };
}

impl OpeningAreaPlateVariants {
    pub fn get_variant(&self, variant: &String) -> &OpeningAreaPlateVariant {
        match variant.as_str() {
            "portal_green_no_glow" => &self.portal_green_no_glow,
            "portal" => &self.portal,
            "green_opener" => &self.green_opener,
            "default" => &self.default,
            "portal_no_glow" => &self.portal_no_glow,
            "portal_green" => &self.portal_green,
            _ => &self.default,
        }
    }
}

pub struct PhysicsOpeningAreaPlate {
    pub platform: PhysicsStaticRigidBody,
    pub activation: PhysicsArea,
}

impl Physical for PhysicsOpeningAreaPlate {
    type Instruction = OpeningAreaPlateVariant;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {
        self.platform.position(physics)
    }

    fn velocity(&self, physics: &RapierSimulation) -> Vector<Real> {
        self.platform.velocity(physics)
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![
            self.platform.collider_handle,
            self.activation.collider_handle,
        ]
    }

    fn create(
        position: Vector<Real>,
        build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        PhysicsOpeningAreaPlate {
            platform: PhysicsStaticRigidBody::create(
                position,
                &build_instructions.shape_platform,
                physics,
            ),
            activation: PhysicsArea::create(
                position,
                &build_instructions.shape_activation,
                physics,
            ),
        }
    }

    fn remove(&self, physics: &mut RapierSimulation) {
        self.platform.remove(physics);
        self.activation.remove(physics);
    }
}

pub struct SlimeChargeVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl SlimeChargeVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct SlimeChargeVariants {
    pub default: SlimeChargeVariant,
}

impl SlimeCharge {
    pub const VARIANTS: SlimeChargeVariants = SlimeChargeVariants {
        default: SlimeChargeVariant {
            gid_default: "535",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl SlimeChargeVariants {
    pub fn get_variant(&self, variant: &String) -> &SlimeChargeVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct SlimeLightVariant {
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl SlimeLightVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct SlimeLightVariants {
    pub default: SlimeLightVariant,
}

impl SlimeLight {
    pub const VARIANTS: SlimeLightVariants = SlimeLightVariants {
        default: SlimeLightVariant {
            gid_default: "383",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl SlimeLightVariants {
    pub fn get_variant(&self, variant: &String) -> &SlimeLightVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct WallPlatformVariant {
    pub gid_off: &'static str,
    pub gid_on: &'static str,
    pub shape_platform: PhysicalShape,
    pub shape_sensor: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl WallPlatformVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct WallPlatformVariants {
    pub default: WallPlatformVariant,
    pub blue: WallPlatformVariant,
    pub red: WallPlatformVariant,
    pub yellow: WallPlatformVariant,
}

impl WallPlatform {
    pub const VARIANTS: WallPlatformVariants = WallPlatformVariants {
        default: WallPlatformVariant {
            gid_off: "957",
            gid_on: "956",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 16.0,
                height: 16.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
            shape_sensor: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 14.0,
                height: 14.0,
            }),
        },
        blue: WallPlatformVariant {
            gid_on: "960",
            gid_off: "961",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 16.0,
                height: 16.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
            shape_sensor: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 14.0,
                height: 14.0,
            }),
        },
        red: WallPlatformVariant {
            gid_on: "958",
            gid_off: "959",
            shape_sensor: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 14.0,
                height: 14.0,
            }),
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 16.0,
                height: 16.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        yellow: WallPlatformVariant {
            gid_on: "962",
            gid_off: "963",
            shape_platform: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 16.0,
                height: 16.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
            shape_sensor: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 14.0,
                height: 14.0,
            }),
        },
    };
}

impl WallPlatformVariants {
    pub fn get_variant(&self, variant: &String) -> &WallPlatformVariant {
        match variant.as_str() {
            "default" => &self.default,
            "blue" => &self.blue,
            "red" => &self.red,
            "yellow" => &self.yellow,
            _ => &self.default,
        }
    }
}

pub struct PhysicsWallPlatform {
    pub sensor: PhysicsArea,
    pub platform: PhysicsStaticRigidBody,
}

impl Physical for PhysicsWallPlatform {
    type Instruction = WallPlatformVariant;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {
        self.platform.position(physics)
    }

    fn velocity(&self, physics: &RapierSimulation) -> Vector<Real> {
        self.platform.velocity(physics)
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![self.sensor.collider_handle, self.platform.collider_handle]
    }

    fn create(
        position: Vector<Real>,
        build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        PhysicsWallPlatform {
            sensor: PhysicsArea::create(position, &build_instructions.shape_sensor, physics),
            platform: PhysicsStaticRigidBody::create(
                position,
                &build_instructions.shape_platform,
                physics,
            ),
        }
    }

    fn remove(&self, physics: &mut RapierSimulation) {
        self.sensor.remove(physics);
        self.platform.remove(physics);
    }
}

pub struct WallPlatformOpenerVariant {
    pub gid_default: &'static str,
    pub gid_off: &'static str,
    pub gid_on: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl WallPlatformOpenerVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct WallPlatformOpenerVariants {
    pub no_heart: WallPlatformOpenerVariant,
    pub yellow: WallPlatformOpenerVariant,
    pub default: WallPlatformOpenerVariant,
    pub blue: WallPlatformOpenerVariant,
}

impl WallPlatformOpener {
    pub const VARIANTS: WallPlatformOpenerVariants = WallPlatformOpenerVariants {
        no_heart: WallPlatformOpenerVariant {
            gid_off: "1000",
            gid_on: "1001",
            gid_default: "999",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        yellow: WallPlatformOpenerVariant {
            gid_on: "973",
            gid_default: "971",
            gid_off: "972",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 6.5455,
                offset_from_center_y: 2.5454998,
                width: 24.0,
                height: 14.0,
            }),
            offset_from_center_x: 6.55,
            offset_from_center_y: 2.55,
        },
        default: WallPlatformOpenerVariant {
            gid_default: "964",
            gid_off: "965",
            gid_on: "966",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 5.0,
                offset_from_center_y: 1.0,
                width: 24.0,
                height: 14.0,
            }),
            offset_from_center_x: 5.00,
            offset_from_center_y: 1.00,
        },
        blue: WallPlatformOpenerVariant {
            gid_on: "980",
            gid_default: "978",
            gid_off: "979",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 6.5455,
                offset_from_center_y: 2.5454998,
                width: 24.0,
                height: 14.0,
            }),
            offset_from_center_x: 6.55,
            offset_from_center_y: 2.55,
        },
    };
}

impl WallPlatformOpenerVariants {
    pub fn get_variant(&self, variant: &String) -> &WallPlatformOpenerVariant {
        match variant.as_str() {
            "no_heart" => &self.no_heart,
            "yellow" => &self.yellow,
            "default" => &self.default,
            "blue" => &self.blue,
            _ => &self.default,
        }
    }
}

pub type BGDecorationEntity = Entity<BGDecoration, PhysicsNone, StaticImage>;
pub type CaveHeartWallEntity = Entity<CaveHeartWall, PhysicsArea, StaticImage>;
pub type DeathAreaEntity = Entity<DeathArea, PhysicsArea, NoRender>;
pub type DeathPropEntity = Entity<DeathProp, PhysicsRigidBody, StaticImage>;
pub type DeathReviveAreaEntity = Entity<DeathReviveArea, PhysicsArea, StaticImage>;
pub type DeathReviveStatueEntity = Entity<DeathReviveStatue, PhysicsArea, StaticImage>;
pub type DeathTorchEntity = Entity<DeathTorch, PhysicsArea, StaticImage>;
pub type DebrisEntity = Entity<Debris, PhysicsNone, StaticImage>;
pub type DoorEntity = Entity<Door, PhysicsStaticRigidBody, StaticImage>;
pub type EnterAreaEntity = Entity<EnterArea, PhysicsArea, NoRender>;
pub type ExitAreaEntity = Entity<ExitArea, PhysicsArea, NoRender>;
pub type GreatWaterfallTilesEntity = Entity<GreatWaterfallTiles, PhysicsNone, StaticImage>;
pub type GuestEntity = Entity<Guest, PhysicsRigidBody, StaticImage>;
pub type GuestDeadEntity = Entity<GuestDead, PhysicsArea, StaticImage>;
pub type GuestNameplateEntity = Entity<GuestNameplate, PhysicsNone, RenderTypeText>;
pub type HeartEntity = Entity<Heart, PhysicsArea, StaticImage>;
pub type HeartSmollEntity = Entity<HeartSmoll, PhysicsNone, StaticImage>;
pub type HeartWallGlowEntity = Entity<HeartWallGlow, PhysicsNone, StaticImage>;
pub type ObserverEntity = Entity<Observer, PhysicsArea, StaticImage>;
pub type OpeningAreaPlateEntity = Entity<OpeningAreaPlate, PhysicsOpeningAreaPlate, StaticImage>;
pub type SecretEntity = Entity<Secret, PhysicsArea, NoRender>;
pub type SlimeChargeEntity = Entity<SlimeCharge, PhysicsNone, StaticImage>;
pub type SlimeLightEntity = Entity<SlimeLight, PhysicsNone, StaticImage>;
pub type TeleportEndEntity = Entity<TeleportEnd, PhysicsArea, NoRender>;
pub type TeleportStartEntity = Entity<TeleportStart, PhysicsArea, NoRender>;
pub type TimerEntity = Entity<Timer, PhysicsNone, RenderTypeTimer>;
pub type WallPlatformEntity = Entity<WallPlatform, PhysicsWallPlatform, StaticImage>;
pub type WallPlatformOpenerEntity = Entity<WallPlatformOpener, PhysicsArea, StaticImage>;

impl BGDecorationEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> BGDecoration {
        BGDecoration {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: BGDecoration,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> BGDecorationEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> BGDecorationEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            BGDecorationEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl CaveHeartWallEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> CaveHeartWall {
        CaveHeartWall {
            blue_heart: obj.get_custom_prop_entity_id("blue_heart"),
            cave_open: obj.get_custom_prop_bool("cave_open"),
            door_to_remove: obj.get_custom_prop_entity_id("door_to_remove"),
            red_heart: obj.get_custom_prop_entity_id("red_heart"),
            yellow_heart: obj.get_custom_prop_entity_id("yellow_heart"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &CaveHeartWallVariant,
        game_state: CaveHeartWall,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> CaveHeartWallEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> CaveHeartWallEntity {
        let physics_instructions =
            CaveHeartWall::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            CaveHeartWallEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DeathAreaEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> DeathArea {
        DeathArea {
            torch: obj.get_custom_prop_entity_id("torch"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: DeathArea,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DeathAreaEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DeathAreaEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DeathAreaEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DeathPropEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> DeathProp {
        DeathProp {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: DeathProp,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DeathPropEntity {
        let physics_body = PhysicsRigidBody::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DeathPropEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DeathPropEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DeathReviveAreaEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> DeathReviveArea {
        DeathReviveArea {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: DeathReviveArea,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DeathReviveAreaEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DeathReviveAreaEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DeathReviveAreaEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DeathReviveStatueEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> DeathReviveStatue {
        DeathReviveStatue {
            revive_area: obj.get_custom_prop_entity_id("revive_area"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &DeathReviveStatueVariant,
        game_state: DeathReviveStatue,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DeathReviveStatueEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DeathReviveStatueEntity {
        let physics_instructions = DeathReviveStatue::VARIANTS
            .get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DeathReviveStatueEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DeathTorchEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> DeathTorch {
        DeathTorch {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &DeathTorchVariant,
        game_state: DeathTorch,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DeathTorchEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DeathTorchEntity {
        let physics_instructions =
            DeathTorch::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DeathTorchEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DebrisEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Debris {
        Debris {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &DebrisVariant,
        game_state: Debris,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DebrisEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DebrisEntity {
        let physics_instructions =
            Debris::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DebrisEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl DoorEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Door {
        Door {
            is_open: obj.get_custom_prop_bool("is_open"),
            open_change_position: obj.get_custom_prop_tween("open_change_position"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &DoorVariant,
        game_state: Door,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> DoorEntity {
        let physics_body =
            PhysicsStaticRigidBody::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> DoorEntity {
        let physics_instructions =
            Door::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            DoorEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl EnterAreaEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> EnterArea {
        EnterArea {
            slot_id: obj.get_custom_prop_string("slot_id"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: EnterArea,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> EnterAreaEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> EnterAreaEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            EnterAreaEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl ExitAreaEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> ExitArea {
        ExitArea {
            slot_id: obj.get_custom_prop_string("slot_id"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: ExitArea,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> ExitAreaEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> ExitAreaEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            ExitAreaEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl GreatWaterfallTilesEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> GreatWaterfallTiles {
        GreatWaterfallTiles {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: GreatWaterfallTiles,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> GreatWaterfallTilesEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> GreatWaterfallTilesEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            GreatWaterfallTilesEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl GuestEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Guest {
        Guest {
            heart_color: obj.get_custom_prop_string("heart_color"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &GuestVariant,
        game_state: Guest,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> GuestEntity {
        let physics_body =
            PhysicsRigidBody::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> GuestEntity {
        let physics_instructions =
            Guest::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            GuestEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl GuestDeadEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> GuestDead {
        GuestDead {
            flame: obj.get_custom_prop_entity_id("flame"),
            heart_color: obj.get_custom_prop_string("heart_color"),
            slime: obj.get_custom_prop_entity_id("slime"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &GuestDeadVariant,
        game_state: GuestDead,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> GuestDeadEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> GuestDeadEntity {
        let physics_instructions =
            GuestDead::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            GuestDeadEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl GuestNameplateEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> GuestNameplate {
        GuestNameplate {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: GuestNameplate,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> GuestNameplateEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: RenderTypeText::from_general_object(&general_object, layer),
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> GuestNameplateEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            GuestNameplateEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl HeartEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Heart {
        Heart {
            color: obj.get_custom_prop_string("color"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &HeartVariant,
        game_state: Heart,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> HeartEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> HeartEntity {
        let physics_instructions =
            Heart::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            HeartEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl HeartSmollEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> HeartSmoll {
        HeartSmoll {
            color: obj.get_custom_prop_string("color"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &HeartSmollVariant,
        game_state: HeartSmoll,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> HeartSmollEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> HeartSmollEntity {
        let physics_instructions =
            HeartSmoll::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            HeartSmollEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl HeartWallGlowEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> HeartWallGlow {
        HeartWallGlow {
            color: obj.get_custom_prop_string("color"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &HeartWallGlowVariant,
        game_state: HeartWallGlow,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> HeartWallGlowEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> HeartWallGlowEntity {
        let physics_instructions =
            HeartWallGlow::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            HeartWallGlowEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl ObserverEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Observer {
        Observer {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &ObserverVariant,
        game_state: Observer,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> ObserverEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> ObserverEntity {
        let physics_instructions =
            Observer::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            ObserverEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl OpeningAreaPlateEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> OpeningAreaPlate {
        OpeningAreaPlate {
            door_to_open_1: obj.get_custom_prop_entity_id("door_to_open_1"),
            door_to_open_2: obj.get_custom_prop_entity_id("door_to_open_2"),
            door_to_open_3: obj.get_custom_prop_entity_id("door_to_open_3"),
            door_to_open_4: obj.get_custom_prop_entity_id("door_to_open_4"),
            opener_variant: obj.get_custom_prop_string("opener_variant"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &OpeningAreaPlateVariant,
        game_state: OpeningAreaPlate,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> OpeningAreaPlateEntity {
        let physics_body = PhysicsOpeningAreaPlate::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> OpeningAreaPlateEntity {
        let physics_instructions = OpeningAreaPlate::VARIANTS
            .get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            OpeningAreaPlateEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl SecretEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Secret {
        Secret {
            secret_name: obj.get_custom_prop_string("secret_name"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Secret,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> SecretEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> SecretEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            SecretEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl SlimeChargeEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> SlimeCharge {
        SlimeCharge {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &SlimeChargeVariant,
        game_state: SlimeCharge,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> SlimeChargeEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> SlimeChargeEntity {
        let physics_instructions =
            SlimeCharge::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            SlimeChargeEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl SlimeLightEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> SlimeLight {
        SlimeLight {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &SlimeLightVariant,
        game_state: SlimeLight,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> SlimeLightEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> SlimeLightEntity {
        let physics_instructions =
            SlimeLight::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            SlimeLightEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl TeleportEndEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> TeleportEnd {
        TeleportEnd {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: TeleportEnd,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> TeleportEndEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> TeleportEndEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            TeleportEndEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl TeleportStartEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> TeleportStart {
        TeleportStart {
            teleport_end: obj.get_custom_prop_entity_id("teleport_end"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: TeleportStart,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> TeleportStartEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> TeleportStartEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            TeleportStartEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl TimerEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Timer {
        Timer {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Timer,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> TimerEntity {
        let physics_body = PhysicsNone::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: RenderTypeTimer::from_general_object(&general_object, layer),
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> TimerEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            TimerEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl WallPlatformEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> WallPlatform {
        WallPlatform {
            off: obj.get_custom_prop_bool("off"),
            opener: obj.get_custom_prop_entity_id("opener"),
            opener_2: obj.get_custom_prop_entity_id("opener_2"),
            opener_3: obj.get_custom_prop_entity_id("opener_3"),
            wall_variant: obj.get_custom_prop_string("wall_variant"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &WallPlatformVariant,
        game_state: WallPlatform,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> WallPlatformEntity {
        let physics_body = PhysicsWallPlatform::create(pos, &physics_instructions, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> WallPlatformEntity {
        let physics_instructions =
            WallPlatform::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            WallPlatformEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl WallPlatformOpenerEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> WallPlatformOpener {
        WallPlatformOpener {
            active: obj.get_custom_prop_bool("active"),
            heart_color: obj.get_custom_prop_string("heart_color"),
            opener_variant: obj.get_custom_prop_string("opener_variant"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &WallPlatformOpenerVariant,
        game_state: WallPlatformOpener,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> WallPlatformOpenerEntity {
        let physics_body = PhysicsArea::create(pos, &physics_instructions.shape_default, physics);

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage {
                width: None,
                height: None,
                tiled: false,
                layer,
                graphic_id,
                blending_mode: None,
                scale: (1.0, 1.0),
                offset_2d: physics_instructions.get_offset_2d(),
            },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> WallPlatformOpenerEntity {
        let physics_instructions = WallPlatformOpener::VARIANTS
            .get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            WallPlatformOpenerEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

pub struct Slime1GameEntityManager {
    pub bg_decoration_map: HashMap<EntityId, BGDecorationEntity>,
    pub cave_heart_wall_map: HashMap<EntityId, CaveHeartWallEntity>,
    pub death_area_map: HashMap<EntityId, DeathAreaEntity>,
    pub death_prop_map: HashMap<EntityId, DeathPropEntity>,
    pub death_revive_area_map: HashMap<EntityId, DeathReviveAreaEntity>,
    pub death_revive_statue_map: HashMap<EntityId, DeathReviveStatueEntity>,
    pub death_torch_map: HashMap<EntityId, DeathTorchEntity>,
    pub debris_map: HashMap<EntityId, DebrisEntity>,
    pub door_map: HashMap<EntityId, DoorEntity>,
    pub enter_area_map: HashMap<EntityId, EnterAreaEntity>,
    pub exit_area_map: HashMap<EntityId, ExitAreaEntity>,
    pub great_waterfall_tiles_map: HashMap<EntityId, GreatWaterfallTilesEntity>,
    pub guest_map: HashMap<EntityId, GuestEntity>,
    pub guest_dead_map: HashMap<EntityId, GuestDeadEntity>,
    pub guest_nameplate_map: HashMap<EntityId, GuestNameplateEntity>,
    pub heart_map: HashMap<EntityId, HeartEntity>,
    pub heart_smoll_map: HashMap<EntityId, HeartSmollEntity>,
    pub heart_wall_glow_map: HashMap<EntityId, HeartWallGlowEntity>,
    pub observer_map: HashMap<EntityId, ObserverEntity>,
    pub opening_area_plate_map: HashMap<EntityId, OpeningAreaPlateEntity>,
    pub secret_map: HashMap<EntityId, SecretEntity>,
    pub slime_charge_map: HashMap<EntityId, SlimeChargeEntity>,
    pub slime_light_map: HashMap<EntityId, SlimeLightEntity>,
    pub teleport_end_map: HashMap<EntityId, TeleportEndEntity>,
    pub teleport_start_map: HashMap<EntityId, TeleportStartEntity>,
    pub timer_map: HashMap<EntityId, TimerEntity>,
    pub wall_platform_map: HashMap<EntityId, WallPlatformEntity>,
    pub wall_platform_opener_map: HashMap<EntityId, WallPlatformOpenerEntity>,
    entity_id_generator: SnowflakeIdBucket,
    pub terrain_map: HashMap<EntityId, TerrainEntity>,
    pub collider_entity_map: ColliderEntityMap<Slime1GameObject>,
    terrain_chunks: Vec<TerrainChunk>,
    guest_id_to_camera_entity_id_map: HashMap<GuestId, EntityId>,
    new_show_entities: Vec<ShowEntity>,
    new_remove_entities: Vec<RemoveEntity>,
    pub new_show_effects: Vec<ShowEffect>,
}

impl Slime1GameEntityManager {
    pub fn new() -> Slime1GameEntityManager {
        Slime1GameEntityManager {
            entity_id_generator: SnowflakeIdBucket::new(1, 5),
            collider_entity_map: ColliderEntityMap::new(),

            terrain_map: HashMap::new(),
            bg_decoration_map: HashMap::new(),
            cave_heart_wall_map: HashMap::new(),
            death_area_map: HashMap::new(),
            death_prop_map: HashMap::new(),
            death_revive_area_map: HashMap::new(),
            death_revive_statue_map: HashMap::new(),
            death_torch_map: HashMap::new(),
            debris_map: HashMap::new(),
            door_map: HashMap::new(),
            enter_area_map: HashMap::new(),
            exit_area_map: HashMap::new(),
            great_waterfall_tiles_map: HashMap::new(),
            guest_map: HashMap::new(),
            guest_dead_map: HashMap::new(),
            guest_nameplate_map: HashMap::new(),
            heart_map: HashMap::new(),
            heart_smoll_map: HashMap::new(),
            heart_wall_glow_map: HashMap::new(),
            observer_map: HashMap::new(),
            opening_area_plate_map: HashMap::new(),
            secret_map: HashMap::new(),
            slime_charge_map: HashMap::new(),
            slime_light_map: HashMap::new(),
            teleport_end_map: HashMap::new(),
            teleport_start_map: HashMap::new(),
            timer_map: HashMap::new(),
            wall_platform_map: HashMap::new(),
            wall_platform_opener_map: HashMap::new(),
            terrain_chunks: Vec::new(),
            guest_id_to_camera_entity_id_map: HashMap::new(),
            new_show_entities: Vec::new(),
            new_remove_entities: Vec::new(),
            new_show_effects: Vec::new(),
        }
    }
    pub fn create_bg_decoration<F: FnMut(&mut BGDecorationEntity)>(
        &mut self,
        game_state: BGDecoration,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = BGDecorationEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::BGDecoration),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.bg_decoration_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_bg_decoration(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<BGDecorationEntity> {
        if let Some(entity) = self.bg_decoration_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_cave_heart_wall<F: FnMut(&mut CaveHeartWallEntity)>(
        &mut self,
        game_state: CaveHeartWall,
        pos: Vector<Real>,
        physics_instructions: &CaveHeartWallVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = CaveHeartWallEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::CaveHeartWall),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.cave_heart_wall_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_cave_heart_wall(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<CaveHeartWallEntity> {
        if let Some(entity) = self.cave_heart_wall_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_death_prop<F: FnMut(&mut DeathPropEntity)>(
        &mut self,
        game_state: DeathProp,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = DeathPropEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::DeathProp),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.death_prop_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_death_prop(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<DeathPropEntity> {
        if let Some(entity) = self.death_prop_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_death_revive_area<F: FnMut(&mut DeathReviveAreaEntity)>(
        &mut self,
        game_state: DeathReviveArea,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = DeathReviveAreaEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::DeathReviveArea),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.death_revive_area_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_death_revive_area(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<DeathReviveAreaEntity> {
        if let Some(entity) = self.death_revive_area_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_death_revive_statue<F: FnMut(&mut DeathReviveStatueEntity)>(
        &mut self,
        game_state: DeathReviveStatue,
        pos: Vector<Real>,
        physics_instructions: &DeathReviveStatueVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = DeathReviveStatueEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::DeathReviveStatue),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.death_revive_statue_map
            .insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_death_revive_statue(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<DeathReviveStatueEntity> {
        if let Some(entity) = self.death_revive_statue_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_death_torch<F: FnMut(&mut DeathTorchEntity)>(
        &mut self,
        game_state: DeathTorch,
        pos: Vector<Real>,
        physics_instructions: &DeathTorchVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = DeathTorchEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::DeathTorch),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.death_torch_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_death_torch(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<DeathTorchEntity> {
        if let Some(entity) = self.death_torch_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_debris<F: FnMut(&mut DebrisEntity)>(
        &mut self,
        game_state: Debris,
        pos: Vector<Real>,
        physics_instructions: &DebrisVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = DebrisEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::Debris),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.debris_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_debris(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<DebrisEntity> {
        if let Some(entity) = self.debris_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_door<F: FnMut(&mut DoorEntity)>(
        &mut self,
        game_state: Door,
        pos: Vector<Real>,
        physics_instructions: &DoorVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = DoorEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map
                .insert(collider_handle, (entity_id.clone(), Slime1GameObject::Door));
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.door_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_door(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<DoorEntity> {
        if let Some(entity) = self.door_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_great_waterfall_tiles<F: FnMut(&mut GreatWaterfallTilesEntity)>(
        &mut self,
        game_state: GreatWaterfallTiles,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = GreatWaterfallTilesEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::GreatWaterfallTiles),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.great_waterfall_tiles_map
            .insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_great_waterfall_tiles(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<GreatWaterfallTilesEntity> {
        if let Some(entity) = self.great_waterfall_tiles_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_guest<F: FnMut(&mut GuestEntity)>(
        &mut self,
        game_state: Guest,
        pos: Vector<Real>,
        physics_instructions: &GuestVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = GuestEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::Guest,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::Guest),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.guest_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_guest(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<GuestEntity> {
        if let Some(entity) = self.guest_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_guest_dead<F: FnMut(&mut GuestDeadEntity)>(
        &mut self,
        game_state: GuestDead,
        pos: Vector<Real>,
        physics_instructions: &GuestDeadVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = GuestDeadEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::GuestDead),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.guest_dead_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_guest_dead(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<GuestDeadEntity> {
        if let Some(entity) = self.guest_dead_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_guest_nameplate<F: FnMut(&mut GuestNameplateEntity)>(
        &mut self,
        game_state: GuestNameplate,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = GuestNameplateEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG0,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::GuestNameplate),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.guest_nameplate_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_guest_nameplate(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<GuestNameplateEntity> {
        if let Some(entity) = self.guest_nameplate_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_heart<F: FnMut(&mut HeartEntity)>(
        &mut self,
        game_state: Heart,
        pos: Vector<Real>,
        physics_instructions: &HeartVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = HeartEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::Heart),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.heart_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_heart(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<HeartEntity> {
        if let Some(entity) = self.heart_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_heart_smoll<F: FnMut(&mut HeartSmollEntity)>(
        &mut self,
        game_state: HeartSmoll,
        pos: Vector<Real>,
        physics_instructions: &HeartSmollVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = HeartSmollEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::HeartSmoll),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.heart_smoll_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_heart_smoll(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<HeartSmollEntity> {
        if let Some(entity) = self.heart_smoll_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_heart_wall_glow<F: FnMut(&mut HeartWallGlowEntity)>(
        &mut self,
        game_state: HeartWallGlow,
        pos: Vector<Real>,
        physics_instructions: &HeartWallGlowVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = HeartWallGlowEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::HeartWallGlow),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.heart_wall_glow_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_heart_wall_glow(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<HeartWallGlowEntity> {
        if let Some(entity) = self.heart_wall_glow_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_observer<F: FnMut(&mut ObserverEntity)>(
        &mut self,
        game_state: Observer,
        pos: Vector<Real>,
        physics_instructions: &ObserverVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = ObserverEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG3,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::Observer),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.observer_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_observer(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<ObserverEntity> {
        if let Some(entity) = self.observer_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_opening_area_plate<F: FnMut(&mut OpeningAreaPlateEntity)>(
        &mut self,
        game_state: OpeningAreaPlate,
        pos: Vector<Real>,
        physics_instructions: &OpeningAreaPlateVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = OpeningAreaPlateEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::OpeningAreaPlate),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.opening_area_plate_map
            .insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_opening_area_plate(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<OpeningAreaPlateEntity> {
        if let Some(entity) = self.opening_area_plate_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_slime_charge<F: FnMut(&mut SlimeChargeEntity)>(
        &mut self,
        game_state: SlimeCharge,
        pos: Vector<Real>,
        physics_instructions: &SlimeChargeVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = SlimeChargeEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::SlimeCharge),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.slime_charge_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_slime_charge(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<SlimeChargeEntity> {
        if let Some(entity) = self.slime_charge_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_slime_light<F: FnMut(&mut SlimeLightEntity)>(
        &mut self,
        game_state: SlimeLight,
        pos: Vector<Real>,
        physics_instructions: &SlimeLightVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = SlimeLightEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::BG0,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::SlimeLight),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.slime_light_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_slime_light(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<SlimeLightEntity> {
        if let Some(entity) = self.slime_light_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_timer<F: FnMut(&mut TimerEntity)>(
        &mut self,
        game_state: Timer,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = TimerEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG3,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::Timer),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.timer_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_timer(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<TimerEntity> {
        if let Some(entity) = self.timer_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_wall_platform<F: FnMut(&mut WallPlatformEntity)>(
        &mut self,
        game_state: WallPlatform,
        pos: Vector<Real>,
        physics_instructions: &WallPlatformVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = WallPlatformEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::WallPlatform),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.wall_platform_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_wall_platform(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<WallPlatformEntity> {
        if let Some(entity) = self.wall_platform_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_wall_platform_opener<F: FnMut(&mut WallPlatformOpenerEntity)>(
        &mut self,
        game_state: WallPlatformOpener,
        pos: Vector<Real>,
        physics_instructions: &WallPlatformOpenerVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = WallPlatformOpenerEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), Slime1GameObject::WallPlatformOpener),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.wall_platform_opener_map
            .insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_wall_platform_opener(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<WallPlatformOpenerEntity> {
        if let Some(entity) = self.wall_platform_opener_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }
}

impl EntityManager for Slime1GameEntityManager {
    fn create_initial(&mut self, map: &TiledMap, physics: &mut RapierSimulation) {
        for layer in &map.layers {
            self.terrain_chunks.extend(layer.terrain_chunks.clone());

            if let LayerName::Terrain = &layer.name {
                let chunks = condense_terrain_from_tiles(layer);
                for chunk in chunks {
                    let (body_handle, collider_handle) =
                        TerrainEntity::create_terrain_collider(&chunk, physics);

                    let terrain_entity = TerrainEntity::new_entity(
                        self.entity_id_generator.get_id().to_string(),
                        Isometry::new(Vector::new(0.0, 0.0), 0.0),
                        body_handle,
                        collider_handle,
                    );
                    self.collider_entity_map.insert(
                        collider_handle,
                        (terrain_entity.id.clone(), Slime1GameObject::Terrain),
                    );
                    self.terrain_map
                        .insert(terrain_entity.id.clone(), terrain_entity);
                }
            }
        }

        let mut reference_map = HashMap::<ObjectId, EntityId>::new();

        for group in &map.object_groups {
            for object in &group.objects {
                if let Ok(kind) = serde_json::from_str(object.kind.as_str()) {
                    let entity_id = self.entity_id_generator.get_id().to_string();
                    reference_map.insert(object.id.clone(), entity_id.clone());

                    match kind {
                        Slime1GameObject::BGDecoration => {
                            let entity = BGDecorationEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::BGDecoration),
                                );
                            }

                            self.bg_decoration_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::CaveHeartWall => {
                            let entity = CaveHeartWallEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::CaveHeartWall),
                                );
                            }

                            self.cave_heart_wall_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::DeathArea => {
                            let entity = DeathAreaEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::DeathArea),
                                );
                            }

                            self.death_area_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::DeathProp => {
                            let entity = DeathPropEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::DeathProp),
                                );
                            }

                            self.death_prop_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::DeathReviveArea => {
                            let entity = DeathReviveAreaEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::DeathReviveArea),
                                );
                            }

                            self.death_revive_area_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::DeathReviveStatue => {
                            let entity = DeathReviveStatueEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::DeathReviveStatue),
                                );
                            }

                            self.death_revive_statue_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::DeathTorch => {
                            let entity = DeathTorchEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::DeathTorch),
                                );
                            }

                            self.death_torch_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Debris => {
                            let entity = DebrisEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Debris),
                                );
                            }

                            self.debris_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Door => {
                            let entity = DoorEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Door),
                                );
                            }

                            self.door_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::EnterArea => {
                            let entity = EnterAreaEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::EnterArea),
                                );
                            }

                            self.enter_area_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::ExitArea => {
                            let entity = ExitAreaEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::ExitArea),
                                );
                            }

                            self.exit_area_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::GreatWaterfallTiles => {
                            let entity = GreatWaterfallTilesEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::GreatWaterfallTiles),
                                );
                            }

                            self.great_waterfall_tiles_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Guest => {
                            let entity = GuestEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Guest),
                                );
                            }

                            self.guest_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::GuestDead => {
                            let entity = GuestDeadEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::GuestDead),
                                );
                            }

                            self.guest_dead_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::GuestNameplate => {
                            let entity = GuestNameplateEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::GuestNameplate),
                                );
                            }

                            self.guest_nameplate_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Heart => {
                            let entity = HeartEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Heart),
                                );
                            }

                            self.heart_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::HeartSmoll => {
                            let entity = HeartSmollEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::HeartSmoll),
                                );
                            }

                            self.heart_smoll_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::HeartWallGlow => {
                            let entity = HeartWallGlowEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::HeartWallGlow),
                                );
                            }

                            self.heart_wall_glow_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Observer => {
                            let entity = ObserverEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Observer),
                                );
                            }

                            self.observer_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::OpeningAreaPlate => {
                            let entity = OpeningAreaPlateEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::OpeningAreaPlate),
                                );
                            }

                            self.opening_area_plate_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Secret => {
                            let entity = SecretEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Secret),
                                );
                            }

                            self.secret_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::SlimeCharge => {
                            let entity = SlimeChargeEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::SlimeCharge),
                                );
                            }

                            self.slime_charge_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::SlimeLight => {
                            let entity = SlimeLightEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::SlimeLight),
                                );
                            }

                            self.slime_light_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::TeleportEnd => {
                            let entity = TeleportEndEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::TeleportEnd),
                                );
                            }

                            self.teleport_end_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::TeleportStart => {
                            let entity = TeleportStartEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::TeleportStart),
                                );
                            }

                            self.teleport_start_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::Timer => {
                            let entity = TimerEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::Timer),
                                );
                            }

                            self.timer_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::WallPlatform => {
                            let entity = WallPlatformEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::WallPlatform),
                                );
                            }

                            self.wall_platform_map.insert(entity_id, entity);
                        }
                        Slime1GameObject::WallPlatformOpener => {
                            let entity = WallPlatformOpenerEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), Slime1GameObject::WallPlatformOpener),
                                );
                            }

                            self.wall_platform_opener_map.insert(entity_id, entity);
                        }
                        kind => {
                            debug!("Not generated right now {:?}", kind);
                        }
                    }
                }
            }
        }

        for entity in self.cave_heart_wall_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.blue_heart) {
                entity.game_state.blue_heart = entity_id.clone();
            }
        }
        for entity in self.cave_heart_wall_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.door_to_remove) {
                entity.game_state.door_to_remove = entity_id.clone();
            }
        }
        for entity in self.cave_heart_wall_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.red_heart) {
                entity.game_state.red_heart = entity_id.clone();
            }
        }
        for entity in self.cave_heart_wall_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.yellow_heart) {
                entity.game_state.yellow_heart = entity_id.clone();
            }
        }
        for entity in self.death_area_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.torch) {
                entity.game_state.torch = entity_id.clone();
            }
        }
        for entity in self.death_revive_statue_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.revive_area) {
                entity.game_state.revive_area = entity_id.clone();
            }
        }
        for entity in self.guest_dead_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.flame) {
                entity.game_state.flame = entity_id.clone();
            }
        }
        for entity in self.guest_dead_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.slime) {
                entity.game_state.slime = entity_id.clone();
            }
        }
        for entity in self.opening_area_plate_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.door_to_open_1) {
                entity.game_state.door_to_open_1 = entity_id.clone();
            }
        }
        for entity in self.opening_area_plate_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.door_to_open_2) {
                entity.game_state.door_to_open_2 = entity_id.clone();
            }
        }
        for entity in self.opening_area_plate_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.door_to_open_3) {
                entity.game_state.door_to_open_3 = entity_id.clone();
            }
        }
        for entity in self.opening_area_plate_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.door_to_open_4) {
                entity.game_state.door_to_open_4 = entity_id.clone();
            }
        }
        for entity in self.teleport_start_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.teleport_end) {
                entity.game_state.teleport_end = entity_id.clone();
            }
        }
        for entity in self.wall_platform_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.opener) {
                entity.game_state.opener = entity_id.clone();
            }
        }
        for entity in self.wall_platform_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.opener_2) {
                entity.game_state.opener_2 = entity_id.clone();
            }
        }
        for entity in self.wall_platform_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.opener_3) {
                entity.game_state.opener_3 = entity_id.clone();
            }
        }
    }

    fn update_entity_positions(&mut self, physics: &mut RapierSimulation) {
        for entity in self.bg_decoration_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.cave_heart_wall_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.death_area_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.death_prop_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.death_revive_area_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.death_revive_statue_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.death_torch_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.debris_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.door_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.enter_area_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.exit_area_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.great_waterfall_tiles_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.guest_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.guest_dead_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.guest_nameplate_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.heart_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.heart_smoll_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.heart_wall_glow_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.observer_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.opening_area_plate_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.secret_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.slime_charge_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.slime_light_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.teleport_end_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.teleport_start_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.timer_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.wall_platform_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.wall_platform_opener_map.values_mut() {
            entity.update_isometry(physics);
        }
    }

    fn set_camera_entity_for_guest(&mut self, guest_id: GuestId, entity_id: EntityId) {
        self.guest_id_to_camera_entity_id_map
            .insert(guest_id, entity_id);
    }

    fn get_current_camera_entity_for_guest(&self, guest_id: &GuestId) -> EntityId {
        self.guest_id_to_camera_entity_id_map
            .get(guest_id)
            .unwrap_or(&String::new())
            .clone()
    }

    fn get_all_terrain_chunks(&mut self) -> Vec<TerrainChunk> {
        self.terrain_chunks.clone()
    }

    fn get_all_show_entities(&mut self) -> Vec<ShowEntity> {
        let mut show_entities = Vec::new();
        for entity in self.bg_decoration_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.cave_heart_wall_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.death_prop_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.death_revive_area_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.death_revive_statue_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.death_torch_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.debris_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.door_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.great_waterfall_tiles_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.guest_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.guest_dead_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.guest_nameplate_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.heart_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.heart_smoll_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.heart_wall_glow_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.observer_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.opening_area_plate_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.slime_charge_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.slime_light_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.timer_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.wall_platform_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.wall_platform_opener_map.values() {
            show_entities.push(entity.show_entity());
        }
        show_entities
    }

    fn get_all_entity_updates(&mut self) -> Vec<UpdateEntity> {
        let mut entity_updates = Vec::new();
        for entity in self.bg_decoration_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.cave_heart_wall_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.death_prop_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.death_revive_area_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.death_revive_statue_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.death_torch_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.debris_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.door_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.great_waterfall_tiles_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.guest_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.guest_dead_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.guest_nameplate_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.heart_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.heart_smoll_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.heart_wall_glow_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.observer_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.opening_area_plate_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.slime_charge_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.slime_light_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.timer_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.wall_platform_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.wall_platform_opener_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        entity_updates
    }

    fn get_all_entity_position_updates(&mut self) -> Vec<(EntityId, Real, Real, Real)> {
        let mut position_updates = Vec::new();
        for entity in self.bg_decoration_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.cave_heart_wall_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.death_prop_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.death_revive_area_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.death_revive_statue_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.death_torch_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.debris_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.door_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.great_waterfall_tiles_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.guest_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.guest_dead_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.guest_nameplate_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.heart_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.heart_smoll_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.heart_wall_glow_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.observer_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.opening_area_plate_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.slime_charge_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.slime_light_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.timer_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.wall_platform_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.wall_platform_opener_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        position_updates
    }

    fn drain_new_show_effects(&mut self) -> Vec<ShowEffect> {
        self.new_show_effects.drain(..).collect()
    }

    fn drain_new_show_entities(&mut self) -> Vec<ShowEntity> {
        self.new_show_entities.drain(..).collect()
    }

    fn drain_new_remove_entities(&mut self) -> Vec<RemoveEntity> {
        self.new_remove_entities.drain(..).collect()
    }
}
