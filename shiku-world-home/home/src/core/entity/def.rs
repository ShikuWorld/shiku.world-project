use crate::core::entity::physics::Physical;
use rapier2d::prelude::{Isometry, Real};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::entity::render::{EntityRenderData, Renderable};
use crate::resource_module::map::def::GeneralObject;

pub type EntityId = String;

#[derive(Debug, Clone)]
pub struct Entity<T, P: Physical, R: Renderable> {
    pub id: EntityId,
    pub isometry: Isometry<Real>,
    pub physics: P,
    pub render: R,
    pub game_state: T,
    pub general_object: Option<GeneralObject>,
    pub is_render_dirty: bool,
    pub is_position_dirty: bool,
    pub parent_entity: Option<EntityId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct ShowEntity {
    pub id: EntityId,
    pub initial_isometrics_2d: (Real, Real, Real),
    pub render: EntityRenderData,
    pub parent_entity: Option<EntityId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct RemoveEntity {
    pub id: EntityId,
    pub parent_entity: Option<EntityId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct UpdateEntity {
    pub id: EntityId,
    pub render: EntityRenderData,
}
