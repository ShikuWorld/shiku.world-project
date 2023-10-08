use crate::core::rapier_simulation::def::RapierSimulation;

use rapier2d::math::Isometry;

use crate::resource_module::map::def::GeneralObject;
use rapier2d::prelude::{ColliderHandle, Real, RigidBodyHandle, Vector};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PhysicsType {
    Area,
    RigidBody,
    StaticRigidBody,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PhysicalShape {
    ShapeRect(ShapeRect),
    ShapePoint(ShapePoint),
    None,
}

impl PhysicalShape {
    pub fn get_offset_2d(&self) -> (Real, Real) {
        match self {
            PhysicalShape::ShapeRect(rect) => {
                (rect.offset_from_center_x, rect.offset_from_center_y)
            }
            PhysicalShape::ShapePoint(point) => {
                (point.offset_from_center_x, point.offset_from_center_y)
            }
            PhysicalShape::None => (0.0, 0.0),
        }
    }
}

pub trait Physical {
    type Instruction;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real>;
    fn velocity(&self, physics: &RapierSimulation) -> Vector<Real>;
    fn get_all_collider_handles(&self) -> Vec<ColliderHandle>;
    fn create(
        position: Vector<Real>,
        build_instruction: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self;
    fn remove(&self, physics: &mut RapierSimulation);
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShapeRect {
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
    pub width: Real,
    pub height: Real,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ShapePoint {
    pub offset_from_center_x: Real,
    pub offset_from_center_y: Real,
}

#[derive(Debug)]
pub struct PhysicsRigidBody {
    pub body_handle: RigidBodyHandle,
    pub velocity: Vector<Real>,
    pub collider_handle: ColliderHandle,
}

#[derive(Debug)]
pub struct PhysicsStaticRigidBody {
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

#[derive(Debug)]
pub struct PhysicsArea {
    pub collider_handle: ColliderHandle,
}

#[derive(Debug)]
pub struct PhysicsNone {
    pub isometry: Isometry<Real>,
}

impl Physical for PhysicsStaticRigidBody {
    type Instruction = PhysicalShape;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {
        if let Some(rigid_body) = physics.bodies.get(self.body_handle) {
            return *rigid_body.position();
        }

        Isometry::identity()
    }

    fn velocity(&self, _physics: &RapierSimulation) -> Vector<Real> {
        Vector::zeros()
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![self.collider_handle]
    }

    fn create(
        position: Vector<Real>,
        build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        match build_instructions {
            PhysicalShape::ShapeRect(rect) => {
                let (body_handle, collider_handle) = physics.add_static_body_cuboid(
                    rect.width / 2.0,
                    rect.height / 2.0,
                    position.x + rect.offset_from_center_x,
                    position.y + rect.offset_from_center_y,
                );

                PhysicsStaticRigidBody {
                    body_handle,
                    collider_handle,
                }
            }
            PhysicalShape::ShapePoint(point) => {
                let (body_handle, collider_handle) = physics.add_static_body_cuboid(
                    1.0,
                    1.0,
                    position.x + point.offset_from_center_x,
                    position.y + point.offset_from_center_y,
                );

                PhysicsStaticRigidBody {
                    body_handle,
                    collider_handle,
                }
            }
            PhysicalShape::None => {
                let (body_handle, collider_handle) =
                    physics.add_static_body_cuboid(1.0, 1.0, position.x, position.y);

                PhysicsStaticRigidBody {
                    body_handle,
                    collider_handle,
                }
            }
        }
    }

    fn remove(&self, physics: &mut RapierSimulation) {
        physics.remove_rigid_body(self.body_handle);
    }
}

impl Physical for PhysicsRigidBody {
    type Instruction = PhysicalShape;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {
        if let Some(rigid_body) = physics.bodies.get(self.body_handle) {
            return *rigid_body.position();
        }

        Isometry::identity()
    }

    fn velocity(&self, physics: &RapierSimulation) -> Vector<Real> {
        if let Some(rigid_body) = physics.bodies.get(self.body_handle) {
            return *rigid_body.linvel();
        }

        Vector::identity()
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![self.collider_handle]
    }

    fn create(
        position: Vector<Real>,
        build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        match build_instructions {
            PhysicalShape::ShapeRect(rect) => {
                let (body_handle, collider_handle) = physics.add_dynamic_body_cuboid(
                    rect.width / 2.0,
                    rect.height / 2.0,
                    position.x + rect.offset_from_center_x,
                    position.y + rect.offset_from_center_y,
                    0.5,
                );

                PhysicsRigidBody {
                    body_handle,
                    collider_handle,
                    velocity: Vector::new(0.0, 0.0),
                }
            }
            PhysicalShape::ShapePoint(point) => {
                let (body_handle, collider_handle) = physics.add_dynamic_body_cuboid(
                    1.0,
                    1.0,
                    position.x + point.offset_from_center_x,
                    position.y + point.offset_from_center_y,
                    0.5,
                );

                PhysicsRigidBody {
                    body_handle,
                    collider_handle,
                    velocity: Vector::new(0.0, 0.0),
                }
            }
            PhysicalShape::None => {
                let (body_handle, collider_handle) =
                    physics.add_dynamic_body_cuboid(1.0, 1.0, position.x, position.y, 0.5);

                PhysicsRigidBody {
                    body_handle,
                    collider_handle,
                    velocity: Vector::new(0.0, 0.0),
                }
            }
        }
    }

    fn remove(&self, physics: &mut RapierSimulation) {
        physics.remove_rigid_body(self.body_handle);
    }
}

impl Physical for PhysicsArea {
    type Instruction = PhysicalShape;

    fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {
        if let Some(collider) = physics.colliders.get(self.collider_handle) {
            return *collider.position();
        }

        Isometry::identity()
    }

    fn velocity(&self, _physics: &RapierSimulation) -> Vector<Real> {
        Vector::zeros()
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![self.collider_handle]
    }

    fn create(
        position: Vector<Real>,
        build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        match build_instructions {
            PhysicalShape::ShapeRect(rect) => {
                let collider_handle = physics.add_static_collision_area(
                    rect.width / 2.0,
                    rect.height / 2.0,
                    position.x + rect.offset_from_center_x,
                    position.y + rect.offset_from_center_y,
                );

                PhysicsArea { collider_handle }
            }
            PhysicalShape::ShapePoint(point) => {
                let collider_handle = physics.add_static_collision_area(
                    1.0,
                    1.0,
                    position.x + point.offset_from_center_x,
                    position.y + point.offset_from_center_y,
                );

                PhysicsArea { collider_handle }
            }
            PhysicalShape::None => {
                let collider_handle =
                    physics.add_static_collision_area(1.0, 1.0, position.x, position.y);

                PhysicsArea { collider_handle }
            }
        }
    }

    fn remove(&self, physics: &mut RapierSimulation) {
        physics.remove_collider(self.collider_handle);
    }
}

impl Physical for PhysicsNone {
    type Instruction = PhysicalShape;

    fn position(&self, _physics: &RapierSimulation) -> Isometry<Real> {
        self.isometry
    }

    fn velocity(&self, _physics: &RapierSimulation) -> Vector<Real> {
        Vector::zeros()
    }

    fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {
        vec![]
    }

    fn create(
        position: Vector<Real>,
        _build_instructions: &Self::Instruction,
        physics: &mut RapierSimulation,
    ) -> Self {
        PhysicsNone {
            isometry: physics.get_isometry_from_world_coordinates(position.x, position.y),
        }
    }

    fn remove(&self, _physics: &mut RapierSimulation) {}
}

impl Display for PhysicsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysicsType::StaticRigidBody => write!(f, "StaticRigidBody"),
            PhysicsType::RigidBody => write!(f, "RigidBody"),
            PhysicsType::Area => write!(f, "Area"),
            PhysicsType::None => write!(f, "None"),
        }
    }
}

impl Display for PhysicalShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysicalShape::ShapeRect(rect) => write!(f, "PhysicalShape::ShapeRect({:?})", rect),
            PhysicalShape::ShapePoint(point) => write!(f, "PhysicalShape::ShapePoint({:?})", point),
            PhysicalShape::None => write!(f, "PhysicalShape::None"),
        }
    }
}

pub fn physical_shape_from_general_obj(obj: &GeneralObject) -> PhysicalShape {
    PhysicalShape::ShapeRect(ShapeRect {
        offset_from_center_x: if obj.graphic_id.is_empty() {
            obj.width / 2.0
        } else {
            0.0
        },
        offset_from_center_y: if obj.graphic_id.is_empty() {
            obj.height / 2.0
        } else {
            0.0
        },
        height: obj.height,
        width: obj.width,
    })
}
