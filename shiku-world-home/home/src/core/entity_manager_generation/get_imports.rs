pub fn get_imports() -> String {
    "#![allow(dead_code)]
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
use std::iter::{Filter, Map};"
        .to_string()
}
