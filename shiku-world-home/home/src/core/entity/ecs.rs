mod engine {
    use crate::core::blueprint;
    use crate::core::Snowflake;
    use rapier2d::dynamics::ImpulseJointHandle;
    use rapier2d::geometry::ColliderHandle;
    use rapier2d::math::Real;

    pub type EntityId = usize;
    pub type JointId = usize;
    pub type Physics = (Real, Real, Real, Real, Real);
    pub type Entity = (
        blueprint::def::EntityId,
        Physics,
        ColliderHandle,
        [Option<Joint>; 32],
        [Option<EntityId>; 32],
    );

    pub struct Guest {
        guest_id: Snowflake,
        persisted_guest_data: EntityId, // Maybe do this through a manager instead?
        entity_id: Option<EntityId>,
    }

    pub struct Joint {
        id: blueprint::def::JointId,
        handle: ImpulseJointHandle,
        collider_a: ColliderHandle,
        collider_b: ColliderHandle,
        used: bool,
    }
}
