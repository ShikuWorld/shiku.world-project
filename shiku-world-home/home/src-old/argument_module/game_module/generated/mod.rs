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
pub enum ArgumentGameObject {
    Terrain,
    Guest,
    EnterArea,
    ExitArea,
    Observer,
    AnxietyBar,
    IceCrack,
    Props,
    Stick,
}

#[derive(Debug)]
pub struct Guest {
    pub flipped: bool,
}

#[derive(Debug)]
pub struct EnterArea {
    pub slot_id: String,
    pub first: bool,
}

#[derive(Debug)]
pub struct ExitArea {
    pub slot_id: String,
}

#[derive(Debug)]
pub struct Observer;

#[derive(Debug)]
pub struct AnxietyBar;

#[derive(Debug)]
pub struct IceCrack;

#[derive(Debug)]
pub struct Props;

#[derive(Debug)]
pub struct Stick {
    pub thrown_by: String,
}

pub struct GuestVariant {
    pub gid_idle_animation_1: &'static str,
    pub gid_hit_charge_1: &'static str,
    pub gid_throw_5: &'static str,
    pub gid_throw_1: &'static str,
    pub gid_hold_throwing: &'static str,
    pub gid_throw_3: &'static str,
    pub gid_hit_charge_2: &'static str,
    pub gid_hit_1: &'static str,
    pub gid_hit_2: &'static str,
    pub gid_walk_forwards: &'static str,
    pub gid_hit_charge_3: &'static str,
    pub gid_: &'static str,
    pub gid_hit_0: &'static str,
    pub gid_walk_backwards: &'static str,
    pub gid_hit_4: &'static str,
    pub gid_start_throwing_2: &'static str,
    pub gid_hit_3: &'static str,
    pub gid_hit_charge_4: &'static str,
    pub gid_throw_4: &'static str,
    pub gid_default: &'static str,
    pub gid_start_throwing_1: &'static str,
    pub gid_throw_2: &'static str,
    pub gid_idle_animation_2: &'static str,
    pub shape_hitbox: PhysicalShape,
    pub shape_default: PhysicalShape,
    pub shape_attack: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl GuestVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct GuestVariants {
    pub default: GuestVariant,
    pub red: GuestVariant,
}

impl Guest {
    pub const VARIANTS: GuestVariants = GuestVariants {
        default: GuestVariant {
            gid_idle_animation_1: "42",
            gid_hit_charge_1: "10",
            gid_throw_5: "121",
            gid_throw_1: "117",
            gid_hold_throwing: "114",
            gid_throw_3: "119",
            gid_hit_charge_2: "11",
            gid_hit_1: "15",
            gid_hit_2: "16",
            gid_walk_forwards: "123",
            gid_hit_charge_3: "12",
            gid_: "131",
            gid_hit_0: "14",
            gid_walk_backwards: "122",
            gid_hit_4: "18",
            gid_start_throwing_2: "113",
            gid_hit_3: "17",
            gid_hit_charge_4: "13",
            gid_throw_4: "120",
            gid_default: "9",
            gid_start_throwing_1: "112",
            gid_throw_2: "118",
            gid_idle_animation_2: "63",
            shape_hitbox: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -2.0,
                offset_from_center_y: 11.0,
                width: 50.0,
                height: 156.0,
            }),
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -2.0,
                offset_from_center_y: 64.0,
                width: 50.0,
                height: 50.0,
            }),
            offset_from_center_x: -2.00,
            offset_from_center_y: 64.00,
            shape_attack: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 95.0,
                offset_from_center_y: 29.0,
                width: 32.0,
                height: 40.0,
            }),
        },
        red: GuestVariant {
            gid_walk_backwards: "258",
            gid_hit_1: "151",
            gid_throw_4: "256",
            gid_throw_1: "253",
            gid_idle_animation_2: "199",
            gid_hit_2: "152",
            gid_idle_animation_1: "178",
            gid_hit_charge_2: "147",
            gid_hit_charge_1: "146",
            gid_: "267",
            gid_hit_4: "154",
            gid_hit_charge_4: "149",
            gid_hit_charge_3: "148",
            gid_start_throwing_1: "248",
            gid_start_throwing_2: "249",
            gid_throw_2: "254",
            gid_throw_5: "257",
            gid_hit_0: "150",
            gid_hold_throwing: "250",
            gid_default: "145",
            gid_hit_3: "153",
            gid_throw_3: "255",
            gid_walk_forwards: "259",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -2.0,
                offset_from_center_y: 64.0,
                width: 50.0,
                height: 50.0,
            }),
            offset_from_center_x: -2.00,
            offset_from_center_y: 64.00,
            shape_hitbox: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -2.0,
                offset_from_center_y: 11.0,
                width: 50.0,
                height: 156.0,
            }),
            shape_attack: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 95.0,
                offset_from_center_y: 29.0,
                width: 32.0,
                height: 40.0,
            }),
        },
    };
}

impl GuestVariants {
    pub fn get_variant(&self, variant: &String) -> &GuestVariant {
        match variant.as_str() {
            "default" => &self.default,
            "red" => &self.red,
            _ => &self.default,
        }
    }
}

pub struct PhysicsGuest {
    pub default: PhysicsRigidBody,
    pub hitbox: PhysicsArea,
    pub attack: PhysicsArea,
}

impl Physical for PhysicsGuest {
    type Instruction = GuestVariant;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {
        self.default.position(physics)
    }

    fn velocity(&self, physics: &RapierSimulation) -> Vector<Real> {
        self.default.velocity(physics)
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![
            self.default.collider_handle,
            self.hitbox.collider_handle,
            self.attack.collider_handle,
        ]
    }

    fn create(
        position: Vector<Real>,
        build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        PhysicsGuest {
            default: PhysicsRigidBody::create(position, &build_instructions.shape_default, physics),
            hitbox: PhysicsArea::create(position, &build_instructions.shape_hitbox, physics),
            attack: PhysicsArea::create(position, &build_instructions.shape_attack, physics),
        }
    }

    fn remove(&self, physics: &mut RapierSimulation) {
        self.default.remove(physics);
        self.hitbox.remove(physics);
        self.attack.remove(physics);
    }
}

pub struct AnxietyBarVariant {
    pub gid_red: &'static str,
    pub gid_default: &'static str,
    pub gid_yellow: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl AnxietyBarVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct AnxietyBarVariants {
    pub default: AnxietyBarVariant,
}

impl AnxietyBar {
    pub const VARIANTS: AnxietyBarVariants = AnxietyBarVariants {
        default: AnxietyBarVariant {
            gid_red: "283",
            gid_default: "281",
            gid_yellow: "282",
            shape_default: PhysicalShape::None,
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
    };
}

impl AnxietyBarVariants {
    pub fn get_variant(&self, variant: &String) -> &AnxietyBarVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub struct IceCrackVariant {
    pub gid_crack: &'static str,
    pub gid_default: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl IceCrackVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct IceCrackVariants {
    pub c3: IceCrackVariant,
    pub default: IceCrackVariant,
    pub c1: IceCrackVariant,
    pub c4: IceCrackVariant,
    pub c2: IceCrackVariant,
}

impl IceCrack {
    pub const VARIANTS: IceCrackVariants = IceCrackVariants {
        c3: IceCrackVariant {
            gid_crack: "369",
            gid_default: "365",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 2.0,
                offset_from_center_y: 10.0,
                width: 56.0,
                height: 140.0,
            }),
            offset_from_center_x: 2.00,
            offset_from_center_y: 10.00,
        },
        default: IceCrackVariant {
            gid_crack: "288",
            gid_default: "284",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -9.0,
                offset_from_center_y: 5.0,
                width: 106.0,
                height: 290.0,
            }),
            offset_from_center_x: -9.00,
            offset_from_center_y: 5.00,
        },
        c1: IceCrackVariant {
            gid_crack: "329",
            gid_default: "325",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -1.0,
                offset_from_center_y: 6.0,
                width: 42.0,
                height: 148.0,
            }),
            offset_from_center_x: -1.00,
            offset_from_center_y: 6.00,
        },
        c4: IceCrackVariant {
            gid_default: "385",
            gid_crack: "389",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -1.0,
                offset_from_center_y: 6.0,
                width: 42.0,
                height: 148.0,
            }),
            offset_from_center_x: -1.00,
            offset_from_center_y: 6.00,
        },
        c2: IceCrackVariant {
            gid_crack: "349",
            gid_default: "345",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -1.0,
                offset_from_center_y: 6.0,
                width: 38.0,
                height: 148.0,
            }),
            offset_from_center_x: -1.00,
            offset_from_center_y: 6.00,
        },
    };
}

impl IceCrackVariants {
    pub fn get_variant(&self, variant: &String) -> &IceCrackVariant {
        match variant.as_str() {
            "c3" => &self.c3,
            "default" => &self.default,
            "c1" => &self.c1,
            "c4" => &self.c4,
            "c2" => &self.c2,
            _ => &self.default,
        }
    }
}

pub struct PropsVariant {
    pub gid_tree_b: &'static str,
    pub gid_trunk_eye: &'static str,
    pub gid_trunk_s: &'static str,
    pub gid_bench_s: &'static str,
    pub gid_fence: &'static str,
    pub gid_lamp: &'static str,
    pub gid_default: &'static str,
    pub gid_tree: &'static str,
    pub gid_trunk: &'static str,
    pub gid_tree_2: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl PropsVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct PropsVariants {
    pub tree: PropsVariant,
    pub tree_b: PropsVariant,
    pub bench_s: PropsVariant,
    pub trunk: PropsVariant,
    pub default: PropsVariant,
    pub trunk_s: PropsVariant,
    pub lamp: PropsVariant,
    pub fence: PropsVariant,
    pub tree_2: PropsVariant,
    pub trunk_eye: PropsVariant,
}

impl Props {
    pub const VARIANTS: PropsVariants = PropsVariants {
        tree: PropsVariant {
            gid_trunk_eye: "313",
            gid_trunk_s: "321",
            gid_fence: "316",
            gid_bench_s: "312",
            gid_tree: "318",
            gid_trunk: "314",
            gid_tree_2: "319",
            gid_tree_b: "320",
            gid_lamp: "315",
            gid_default: "311",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 34.0,
                offset_from_center_y: 245.0,
                width: 192.0,
                height: 152.0,
            }),
            offset_from_center_x: 34.00,
            offset_from_center_y: 245.00,
        },
        tree_b: PropsVariant {
            gid_trunk_eye: "315",
            gid_default: "313",
            gid_trunk_s: "323",
            gid_lamp: "317",
            gid_fence: "318",
            gid_tree: "320",
            gid_trunk: "316",
            gid_tree_b: "322",
            gid_tree_2: "321",
            gid_bench_s: "314",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 17.5,
                offset_from_center_y: -3.0,
                width: 412.0,
                height: 182.0,
            }),
            offset_from_center_x: 17.50,
            offset_from_center_y: -3.00,
        },
        bench_s: PropsVariant {
            gid_trunk_eye: "307",
            gid_trunk: "308",
            gid_tree_b: "314",
            gid_bench_s: "306",
            gid_tree: "312",
            gid_tree_2: "313",
            gid_lamp: "309",
            gid_trunk_s: "315",
            gid_default: "305",
            gid_fence: "310",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -6.0,
                offset_from_center_y: -6.0,
                width: 276.0,
                height: 66.0,
            }),
            offset_from_center_x: -6.00,
            offset_from_center_y: -6.00,
        },
        trunk: PropsVariant {
            gid_tree: "314",
            gid_tree_b: "316",
            gid_lamp: "311",
            gid_default: "307",
            gid_trunk_eye: "309",
            gid_trunk_s: "317",
            gid_tree_2: "315",
            gid_bench_s: "308",
            gid_trunk: "310",
            gid_fence: "312",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -18.5,
                offset_from_center_y: 69.0,
                width: 394.0,
                height: 104.0,
            }),
            offset_from_center_x: -18.50,
            offset_from_center_y: 69.00,
        },
        default: PropsVariant {
            gid_tree_b: "313",
            gid_trunk_eye: "306",
            gid_trunk_s: "314",
            gid_bench_s: "305",
            gid_fence: "309",
            gid_lamp: "308",
            gid_default: "304",
            gid_tree: "311",
            gid_trunk: "307",
            gid_tree_2: "312",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 20.0,
                width: 276.0,
                height: 64.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 20.00,
        },
        trunk_s: PropsVariant {
            gid_tree_2: "322",
            gid_trunk: "317",
            gid_bench_s: "315",
            gid_tree: "321",
            gid_trunk_eye: "316",
            gid_tree_b: "323",
            gid_default: "314",
            gid_trunk_s: "324",
            gid_lamp: "318",
            gid_fence: "319",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 16.5,
                offset_from_center_y: 11.0,
                width: 154.0,
                height: 60.0,
            }),
            offset_from_center_x: 16.50,
            offset_from_center_y: 11.00,
        },
        lamp: PropsVariant {
            gid_bench_s: "309",
            gid_trunk_eye: "310",
            gid_default: "308",
            gid_trunk: "311",
            gid_lamp: "312",
            gid_tree_2: "316",
            gid_tree_b: "317",
            gid_trunk_s: "318",
            gid_fence: "313",
            gid_tree: "315",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 16.5,
                offset_from_center_y: 202.5,
                width: 86.0,
                height: 86.0,
            }),
            offset_from_center_x: 16.50,
            offset_from_center_y: 202.50,
        },
        fence: PropsVariant {
            gid_trunk_eye: "311",
            gid_tree_b: "318",
            gid_bench_s: "310",
            gid_trunk_s: "319",
            gid_fence: "314",
            gid_lamp: "313",
            gid_tree: "316",
            gid_default: "309",
            gid_trunk: "312",
            gid_tree_2: "317",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 0.0,
                offset_from_center_y: 0.0,
                width: 1920.0,
                height: 127.0,
            }),
            offset_from_center_x: 0.00,
            offset_from_center_y: 0.00,
        },
        tree_2: PropsVariant {
            gid_tree: "319",
            gid_tree_b: "321",
            gid_trunk_eye: "314",
            gid_fence: "317",
            gid_default: "312",
            gid_tree_2: "320",
            gid_bench_s: "313",
            gid_trunk: "315",
            gid_trunk_s: "322",
            gid_lamp: "316",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 10.5,
                offset_from_center_y: 223.0,
                width: 112.0,
                height: 74.0,
            }),
            offset_from_center_x: 10.50,
            offset_from_center_y: 223.00,
        },
        trunk_eye: PropsVariant {
            gid_tree_b: "315",
            gid_trunk: "309",
            gid_tree_2: "314",
            gid_bench_s: "307",
            gid_fence: "311",
            gid_lamp: "310",
            gid_trunk_eye: "308",
            gid_default: "306",
            gid_tree: "313",
            gid_trunk_s: "316",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: -18.5,
                offset_from_center_y: 69.0,
                width: 394.0,
                height: 104.0,
            }),
            offset_from_center_x: -18.50,
            offset_from_center_y: 69.00,
        },
    };
}

impl PropsVariants {
    pub fn get_variant(&self, variant: &String) -> &PropsVariant {
        match variant.as_str() {
            "tree" => &self.tree,
            "tree_b" => &self.tree_b,
            "bench_s" => &self.bench_s,
            "trunk" => &self.trunk,
            "default" => &self.default,
            "trunk_s" => &self.trunk_s,
            "lamp" => &self.lamp,
            "fence" => &self.fence,
            "tree_2" => &self.tree_2,
            "trunk_eye" => &self.trunk_eye,
            _ => &self.default,
        }
    }
}

pub struct StickVariant {
    pub gid_default: &'static str,
    pub gid_backwards: &'static str,
    pub shape_default: PhysicalShape,
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

impl StickVariant {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        (self.offset_from_center_x, self.offset_from_center_y)
    }
}

pub struct StickVariants {
    pub default: StickVariant,
}

impl Stick {
    pub const VARIANTS: StickVariants = StickVariants {
        default: StickVariant {
            gid_default: "315",
            gid_backwards: "316",
            shape_default: PhysicalShape::ShapeRect(ShapeRect {
                offset_from_center_x: 3.0,
                offset_from_center_y: -1.0,
                width: 50.0,
                height: 50.0,
            }),
            offset_from_center_x: 3.00,
            offset_from_center_y: -1.00,
        },
    };
}

impl StickVariants {
    pub fn get_variant(&self, variant: &String) -> &StickVariant {
        match variant.as_str() {
            "default" => &self.default,
            _ => &self.default,
        }
    }
}

pub type GuestEntity = Entity<Guest, PhysicsGuest, StaticImage>;
pub type EnterAreaEntity = Entity<EnterArea, PhysicsArea, NoRender>;
pub type ExitAreaEntity = Entity<ExitArea, PhysicsArea, NoRender>;
pub type ObserverEntity = Entity<Observer, PhysicsArea, StaticImage>;
pub type AnxietyBarEntity = Entity<AnxietyBar, PhysicsNone, StaticImage>;
pub type IceCrackEntity = Entity<IceCrack, PhysicsArea, StaticImage>;
pub type PropsEntity = Entity<Props, PhysicsStaticRigidBody, StaticImage>;
pub type StickEntity = Entity<Stick, PhysicsArea, StaticImage>;

impl GuestEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Guest {
        Guest {
            flipped: obj.get_custom_prop_bool("flipped"),
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
        let physics_body = PhysicsGuest::create(pos, &physics_instructions, physics);

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

impl EnterAreaEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> EnterArea {
        EnterArea {
            slot_id: obj.get_custom_prop_string("slot_id"),
            first: obj.get_custom_prop_bool("first"),
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

impl ObserverEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Observer {
        Observer {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Observer,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> ObserverEntity {
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
    ) -> ObserverEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

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

impl AnxietyBarEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> AnxietyBar {
        AnxietyBar {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &AnxietyBarVariant,
        game_state: AnxietyBar,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> AnxietyBarEntity {
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
    ) -> AnxietyBarEntity {
        let physics_instructions =
            AnxietyBar::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            AnxietyBarEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl IceCrackEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> IceCrack {
        IceCrack {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &IceCrackVariant,
        game_state: IceCrack,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> IceCrackEntity {
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
    ) -> IceCrackEntity {
        let physics_instructions =
            IceCrack::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            IceCrackEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl PropsEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Props {
        Props {}
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PropsVariant,
        game_state: Props,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> PropsEntity {
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
    ) -> PropsEntity {
        let physics_instructions =
            Props::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            PropsEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

impl StickEntity {
    pub fn game_state_from_general_object(obj: &GeneralObject) -> Stick {
        Stick {
            thrown_by: obj.get_custom_prop_string("thrown_by"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &StickVariant,
        game_state: Stick,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> StickEntity {
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
    ) -> StickEntity {
        let physics_instructions =
            Stick::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            StickEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }
}

pub struct ArgumentGameEntityManager {
    pub guest_map: HashMap<EntityId, GuestEntity>,
    pub enter_area_map: HashMap<EntityId, EnterAreaEntity>,
    pub exit_area_map: HashMap<EntityId, ExitAreaEntity>,
    pub observer_map: HashMap<EntityId, ObserverEntity>,
    pub anxiety_bar_map: HashMap<EntityId, AnxietyBarEntity>,
    pub ice_crack_map: HashMap<EntityId, IceCrackEntity>,
    pub props_map: HashMap<EntityId, PropsEntity>,
    pub stick_map: HashMap<EntityId, StickEntity>,
    entity_id_generator: SnowflakeIdBucket,
    pub terrain_map: HashMap<EntityId, TerrainEntity>,
    pub collider_entity_map: ColliderEntityMap<ArgumentGameObject>,
    terrain_chunks: Vec<TerrainChunk>,
    guest_id_to_camera_entity_id_map: HashMap<GuestId, EntityId>,
    new_show_entities: Vec<ShowEntity>,
    new_remove_entities: Vec<RemoveEntity>,
    pub new_show_effects: Vec<ShowEffect>,
}

impl ArgumentGameEntityManager {
    pub fn new() -> ArgumentGameEntityManager {
        ArgumentGameEntityManager {
            entity_id_generator: SnowflakeIdBucket::new(1, 2),
            collider_entity_map: ColliderEntityMap::new(),

            terrain_map: HashMap::new(),
            guest_map: HashMap::new(),
            enter_area_map: HashMap::new(),
            exit_area_map: HashMap::new(),
            observer_map: HashMap::new(),
            anxiety_bar_map: HashMap::new(),
            ice_crack_map: HashMap::new(),
            props_map: HashMap::new(),
            stick_map: HashMap::new(),
            terrain_chunks: Vec::new(),
            guest_id_to_camera_entity_id_map: HashMap::new(),
            new_show_entities: Vec::new(),
            new_remove_entities: Vec::new(),
            new_show_effects: Vec::new(),
        }
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
            LayerName::GameObjects,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), ArgumentGameObject::Guest),
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

    pub fn create_observer<F: FnMut(&mut ObserverEntity)>(
        &mut self,
        game_state: Observer,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
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
                (entity_id.clone(), ArgumentGameObject::Observer),
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

    pub fn create_anxiety_bar<F: FnMut(&mut AnxietyBarEntity)>(
        &mut self,
        game_state: AnxietyBar,
        pos: Vector<Real>,
        physics_instructions: &AnxietyBarVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = AnxietyBarEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG5,
            physics_instructions,
            game_state,
            None,
            physics,
        );

        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), ArgumentGameObject::AnxietyBar),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.anxiety_bar_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_anxiety_bar(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<AnxietyBarEntity> {
        if let Some(entity) = self.anxiety_bar_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_ice_crack<F: FnMut(&mut IceCrackEntity)>(
        &mut self,
        game_state: IceCrack,
        pos: Vector<Real>,
        physics_instructions: &IceCrackVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = IceCrackEntity::new(
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
                (entity_id.clone(), ArgumentGameObject::IceCrack),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.ice_crack_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_ice_crack(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<IceCrackEntity> {
        if let Some(entity) = self.ice_crack_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_props<F: FnMut(&mut PropsEntity)>(
        &mut self,
        game_state: Props,
        pos: Vector<Real>,
        physics_instructions: &PropsVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = PropsEntity::new(
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
                (entity_id.clone(), ArgumentGameObject::Props),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.props_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_props(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<PropsEntity> {
        if let Some(entity) = self.props_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

    pub fn create_stick<F: FnMut(&mut StickEntity)>(
        &mut self,
        game_state: Stick,
        pos: Vector<Real>,
        physics_instructions: &StickVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F,
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = StickEntity::new(
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
                (entity_id.clone(), ArgumentGameObject::Stick),
            );
        }

        adjust_callback(&mut entity);

        self.new_show_entities.push(entity.show_entity());

        self.stick_map.insert(entity_id.clone(), entity);

        entity_id
    }

    pub fn remove_stick(
        &mut self,
        entity_id: &EntityId,
        physics: &mut RapierSimulation,
    ) -> Option<StickEntity> {
        if let Some(entity) = self.stick_map.remove(entity_id) {
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

impl EntityManager for ArgumentGameEntityManager {
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
                        (terrain_entity.id.clone(), ArgumentGameObject::Terrain),
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
                        ArgumentGameObject::Guest => {
                            let entity = GuestEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::Guest),
                                );
                            }

                            self.guest_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::EnterArea => {
                            let entity = EnterAreaEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::EnterArea),
                                );
                            }

                            self.enter_area_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::ExitArea => {
                            let entity = ExitAreaEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::ExitArea),
                                );
                            }

                            self.exit_area_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::Observer => {
                            let entity = ObserverEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::Observer),
                                );
                            }

                            self.observer_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::AnxietyBar => {
                            let entity = AnxietyBarEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::AnxietyBar),
                                );
                            }

                            self.anxiety_bar_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::IceCrack => {
                            let entity = IceCrackEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::IceCrack),
                                );
                            }

                            self.ice_crack_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::Props => {
                            let entity = PropsEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::Props),
                                );
                            }

                            self.props_map.insert(entity_id, entity);
                        }
                        ArgumentGameObject::Stick => {
                            let entity = StickEntity::new_from_general_object(
                                entity_id.clone(),
                                object,
                                group.layer_name.clone(),
                                physics,
                            );

                            for collider_handle in entity.physics.get_all_collider_handles() {
                                self.collider_entity_map.insert(
                                    collider_handle,
                                    (entity_id.clone(), ArgumentGameObject::Stick),
                                );
                            }

                            self.stick_map.insert(entity_id, entity);
                        }
                        kind => {
                            debug!("Not generated right now {:?}", kind);
                        }
                    }
                }
            }
        }
    }

    fn update_entity_positions(&mut self, physics: &mut RapierSimulation) {
        for entity in self.guest_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.enter_area_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.exit_area_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.observer_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.anxiety_bar_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.ice_crack_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.props_map.values_mut() {
            entity.update_isometry(physics);
        }
        for entity in self.stick_map.values_mut() {
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
        for entity in self.guest_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.observer_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.anxiety_bar_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.ice_crack_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.props_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.stick_map.values() {
            show_entities.push(entity.show_entity());
        }
        show_entities
    }

    fn get_all_entity_updates(&mut self) -> Vec<UpdateEntity> {
        let mut entity_updates = Vec::new();
        for entity in self.guest_map.values_mut() {
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
        for entity in self.anxiety_bar_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.ice_crack_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.props_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.stick_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        entity_updates
    }

    fn get_all_entity_position_updates(&mut self) -> Vec<(EntityId, Real, Real, Real)> {
        let mut position_updates = Vec::new();
        for entity in self.guest_map.values_mut() {
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
        for entity in self.anxiety_bar_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.ice_crack_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.props_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.stick_map.values_mut() {
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
