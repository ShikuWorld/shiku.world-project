use crate::slime_1_module::game_module::generated::{Guest, GuestVariant};

use rapier2d::prelude::Real;

use crate::core::animation::{Animation, AnimationFrame};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Instant;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum GuestMovementState {
    Idle,
    Facing,
    Extending,
    Moved,
    HoldJump,
    ReleaseJump,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GuestDirections {
    Left,
    Right,
    Up,
}

pub struct GuestMovement {
    pub skin_name: String,
    pub state: GuestMovementState,
    pub animation_states: HashMap<GuestMovementState, Animation<GuestDirections>>,
    current_direction: GuestDirections,
    pub time_of_last_direction_change: Instant,
    pub charged_effect_shown: bool,
    time_in_current_state_in_ms: Real,
    pub jumped_in_extended: bool,
}

impl GuestMovement {
    pub fn get_current_direction(&self) -> &GuestDirections {
        &self.current_direction
    }

    pub fn change_direction(&mut self, direction: GuestDirections) {
        self.time_of_last_direction_change = Instant::now();
        self.current_direction = direction;
    }

    pub fn ms_since_last_direction_change(&mut self) -> u128 {
        Instant::now()
            .duration_since(self.time_of_last_direction_change)
            .as_millis()
    }

    pub fn new(skin_name: &String) -> GuestMovement {
        let slime_skin: &GuestVariant = Guest::VARIANTS.get_variant(skin_name);

        GuestMovement {
            skin_name: skin_name.clone(),
            state: GuestMovementState::Facing,
            time_in_current_state_in_ms: 0.0,
            time_of_last_direction_change: Instant::now(),
            current_direction: GuestDirections::Right,
            jumped_in_extended: false,
            charged_effect_shown: false,
            animation_states: [
                (
                    GuestMovementState::Idle,
                    Animation::new(vec![
                        AnimationFrame::new(
                            250.0,
                            HashMap::from([(GuestDirections::Up, slime_skin.gid_idle_1)]),
                        ),
                        AnimationFrame::new(
                            250.0,
                            HashMap::from([(GuestDirections::Up, slime_skin.gid_idle_2)]),
                        ),
                        AnimationFrame::new(
                            250.0,
                            HashMap::from([(GuestDirections::Up, slime_skin.gid_idle_3)]),
                        ),
                        AnimationFrame::new(
                            250.0,
                            HashMap::from([(GuestDirections::Up, slime_skin.gid_idle_4)]),
                        ),
                    ]),
                ),
                (
                    GuestMovementState::Facing,
                    Animation::new(vec![AnimationFrame::new(
                        0.0,
                        HashMap::from([
                            (GuestDirections::Left, slime_skin.gid_face_left),
                            (GuestDirections::Right, slime_skin.gid_face_right),
                            (GuestDirections::Up, slime_skin.gid_jump_hold_up_1),
                        ]),
                    )]),
                ),
                (
                    GuestMovementState::Extending,
                    Animation::new(vec![
                        AnimationFrame::new(
                            350.0,
                            HashMap::from([
                                (GuestDirections::Left, slime_skin.gid_extend_left_1),
                                (GuestDirections::Right, slime_skin.gid_extend_right_1),
                            ]),
                        ),
                        AnimationFrame::new(
                            0.0,
                            HashMap::from([
                                (GuestDirections::Left, slime_skin.gid_extend_left_2),
                                (GuestDirections::Right, slime_skin.gid_extend_right_2),
                            ]),
                        ),
                    ]),
                ),
                (
                    GuestMovementState::Moved,
                    Animation::new(vec![AnimationFrame::new(
                        200.0,
                        HashMap::from([
                            (GuestDirections::Left, slime_skin.gid_moved_left),
                            (GuestDirections::Right, slime_skin.gid_moved_right),
                        ]),
                    )]),
                ),
                (
                    GuestMovementState::HoldJump,
                    Animation::new(vec![
                        AnimationFrame::new(
                            350.0,
                            HashMap::from([
                                (GuestDirections::Left, slime_skin.gid_jump_hold_left_1),
                                (GuestDirections::Right, slime_skin.gid_jump_hold_right_1),
                                (GuestDirections::Up, slime_skin.gid_jump_hold_up_1),
                            ]),
                        ),
                        AnimationFrame::new(
                            350.0,
                            HashMap::from([
                                (GuestDirections::Left, slime_skin.gid_jump_hold_left_2),
                                (GuestDirections::Right, slime_skin.gid_jump_hold_right_2),
                                (GuestDirections::Up, slime_skin.gid_jump_hold_up_2),
                            ]),
                        ),
                        AnimationFrame::new(
                            0.0,
                            HashMap::from([
                                (GuestDirections::Left, slime_skin.gid_jump_hold_left_3),
                                (GuestDirections::Right, slime_skin.gid_jump_hold_right_3),
                                (GuestDirections::Up, slime_skin.gid_jump_hold_up_3),
                            ]),
                        ),
                    ]),
                ),
                (
                    GuestMovementState::ReleaseJump,
                    Animation::new(vec![AnimationFrame::new(
                        250.0,
                        HashMap::from([
                            (GuestDirections::Left, slime_skin.gid_jumping_left),
                            (GuestDirections::Right, slime_skin.gid_jumping_right),
                            (GuestDirections::Up, slime_skin.gid_jumping_up),
                        ]),
                    )]),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }

    pub fn get_current_gid(&self) -> &'static str {
        if let Some(animation) = self.animation_states.get(&self.state) {
            return animation.get_current_gid(&self.current_direction);
        }

        "1"
    }

    pub fn is_current_animation_done(&self) -> bool {
        if let Some(animation) = self.animation_states.get(&self.state) {
            return animation.done;
        }

        false
    }

    pub fn current_animation_progress(&self) -> Real {
        if let Some(animation) = self.animation_states.get(&self.state) {
            return animation.progress();
        }

        0.0
    }

    pub fn cancel_move(&mut self) {
        if self.state == GuestMovementState::Extending {
            self.state = GuestMovementState::Facing
        }
    }

    pub fn move_in_direction(&mut self) {
        match self.state {
            GuestMovementState::Idle => self.state = GuestMovementState::Extending,
            GuestMovementState::Facing => self.state = GuestMovementState::Extending,
            GuestMovementState::Extending => self.state = GuestMovementState::Moved,
            GuestMovementState::Moved => self.state = GuestMovementState::Facing,
            _ => (),
        }

        self.start_animation();
    }

    pub fn afk(&mut self) {
        self.state = GuestMovementState::Idle;
        self.current_direction = GuestDirections::Up;

        self.start_animation();
    }

    pub fn advance_jumping(&mut self) {
        match self.state {
            GuestMovementState::Idle => self.state = GuestMovementState::HoldJump,
            GuestMovementState::Facing => self.state = GuestMovementState::HoldJump,
            GuestMovementState::HoldJump => self.state = GuestMovementState::ReleaseJump,
            GuestMovementState::ReleaseJump => self.state = GuestMovementState::Facing,
            _ => (),
        }

        self.start_animation();
    }

    fn start_animation(&mut self) {
        if let Some(animation) = self.animation_states.get_mut(&self.state) {
            self.time_in_current_state_in_ms = 0.0;
            self.charged_effect_shown = false;
            animation.start();
        }
    }

    pub fn update(&mut self, update_time: Real) {
        if let Some(animation) = self.animation_states.get_mut(&self.state) {
            animation.run(update_time);
            self.time_in_current_state_in_ms += update_time;
        }
    }
}
