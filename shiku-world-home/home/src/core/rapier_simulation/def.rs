use rapier2d::prelude::*;

pub struct RapierSimulation {
    pub(crate) gravity: Vector<Real>,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) islands: IslandManager,
    pub(crate) broad_phase_multi_sap: BroadPhaseMultiSap,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) query_pipeline: QueryPipeline,
    pub(crate) physics_hooks: (),
    pub(crate) events: Box<dyn EventHandler>,
    pub(crate) physics_pipeline: PhysicsPipeline,
}
