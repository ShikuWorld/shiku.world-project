use rapier2d::prelude::Real;

use crate::argument_module::game_module::generated::{Guest, GuestVariant};
use crate::core::animation::{Animation, AnimationFrame};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Instant;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum CharacterMovementState {
    Idle,
    Idle2,
    MoveForwards,
    MoveBackwards,
    StartThrowing,
    HoldThrow,
    Throw,
    Hitting,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CharacterDirections {
    Left,
    Right,
}

pub struct CharacterMovement {
    pub skin_name: String,
    pub state: CharacterMovementState,
    pub animation_states: HashMap<CharacterMovementState, Animation<CharacterDirections>>,
    current_direction: CharacterDirections,
    time_in_current_state_in_ms: Real,
}

impl CharacterMovement {
    pub fn get_current_direction(&self) -> &CharacterDirections {
        &self.current_direction
    }

    pub fn change_direction(&mut self, direction: CharacterDirections) {
        self.current_direction = direction;
    }

    pub fn new(skin_name: &String) -> CharacterMovement {
        let char_skin: &GuestVariant = Guest::VARIANTS.get_variant(skin_name);

        CharacterMovement {
            skin_name: skin_name.clone(),
            state: CharacterMovementState::Idle,
            time_in_current_state_in_ms: 0.0,
            current_direction: CharacterDirections::Right,
            animation_states: [
                (
                    CharacterMovementState::Hitting,
                    Animation::new(vec![
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_charge_1),
                                (CharacterDirections::Left, char_skin.gid_hit_charge_1),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_charge_2),
                                (CharacterDirections::Left, char_skin.gid_hit_charge_2),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_charge_3),
                                (CharacterDirections::Left, char_skin.gid_hit_charge_3),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_charge_4),
                                (CharacterDirections::Left, char_skin.gid_hit_charge_4),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_0),
                                (CharacterDirections::Left, char_skin.gid_hit_0),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_1),
                                (CharacterDirections::Left, char_skin.gid_hit_1),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_2),
                                (CharacterDirections::Left, char_skin.gid_hit_2),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_3),
                                (CharacterDirections::Left, char_skin.gid_hit_3),
                            ]),
                        ),
                        AnimationFrame::new(
                            66.6,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_hit_4),
                                (CharacterDirections::Left, char_skin.gid_hit_4),
                            ]),
                        ),
                    ]),
                ),
                (
                    CharacterMovementState::StartThrowing,
                    Animation::new(vec![
                        AnimationFrame::new(
                            200.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_start_throwing_1),
                                (CharacterDirections::Left, char_skin.gid_start_throwing_1),
                            ]),
                        ),
                        AnimationFrame::new(
                            150.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_start_throwing_2),
                                (CharacterDirections::Left, char_skin.gid_start_throwing_2),
                            ]),
                        ),
                    ]),
                ),
                (
                    CharacterMovementState::HoldThrow,
                    Animation::new(vec![AnimationFrame::new(
                        300.0,
                        HashMap::from([
                            (CharacterDirections::Right, char_skin.gid_hold_throwing),
                            (CharacterDirections::Left, char_skin.gid_hold_throwing),
                        ]),
                    )]),
                ),
                (
                    CharacterMovementState::Throw,
                    Animation::new(vec![
                        AnimationFrame::new(
                            150.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_throw_1),
                                (CharacterDirections::Left, char_skin.gid_throw_1),
                            ]),
                        ),
                        AnimationFrame::new(
                            150.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_throw_2),
                                (CharacterDirections::Left, char_skin.gid_throw_2),
                            ]),
                        ),
                        AnimationFrame::new(
                            100.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_throw_3),
                                (CharacterDirections::Left, char_skin.gid_throw_3),
                            ]),
                        ),
                        AnimationFrame::new(
                            100.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_throw_4),
                                (CharacterDirections::Left, char_skin.gid_throw_4),
                            ]),
                        ),
                        AnimationFrame::new(
                            100.0,
                            HashMap::from([
                                (CharacterDirections::Right, char_skin.gid_throw_5),
                                (CharacterDirections::Left, char_skin.gid_throw_5),
                            ]),
                        ),
                    ]),
                ),
                (
                    CharacterMovementState::Idle,
                    Animation::new(vec![AnimationFrame::new(
                        250.0,
                        HashMap::from([
                            (CharacterDirections::Right, char_skin.gid_idle_animation_1),
                            (CharacterDirections::Left, char_skin.gid_idle_animation_1),
                        ]),
                    )]),
                ),
                (
                    CharacterMovementState::MoveForwards,
                    Animation::new(vec![AnimationFrame::new(
                        250.0,
                        HashMap::from([
                            (CharacterDirections::Right, char_skin.gid_walk_forwards),
                            (CharacterDirections::Left, char_skin.gid_walk_forwards),
                        ]),
                    )]),
                ),
                (
                    CharacterMovementState::MoveBackwards,
                    Animation::new(vec![AnimationFrame::new(
                        250.0,
                        HashMap::from([
                            (CharacterDirections::Right, char_skin.gid_walk_backwards),
                            (CharacterDirections::Left, char_skin.gid_walk_backwards),
                        ]),
                    )]),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }

    pub fn move_forwards(&mut self) {
        match self.state {
            CharacterMovementState::Idle => self.state = CharacterMovementState::MoveForwards,
            CharacterMovementState::MoveBackwards => {
                self.state = CharacterMovementState::MoveForwards
            }
            _ => (),
        }

        self.start_animation();
    }

    pub fn hit(&mut self) {
        match self.state {
            CharacterMovementState::Idle => self.state = CharacterMovementState::Hitting,
            CharacterMovementState::MoveForwards => self.state = CharacterMovementState::Hitting,
            CharacterMovementState::MoveBackwards => self.state = CharacterMovementState::Hitting,
            _ => (),
        }

        self.start_animation();
    }

    pub fn hit_done(&mut self) {
        match self.state {
            CharacterMovementState::Hitting => self.state = CharacterMovementState::Idle,
            _ => (),
        }

        self.start_animation();
    }

    pub fn start_throw(&mut self) {
        match self.state {
            CharacterMovementState::Idle => self.state = CharacterMovementState::StartThrowing,
            CharacterMovementState::MoveForwards => {
                self.state = CharacterMovementState::StartThrowing
            }
            _ => (),
        }

        self.start_animation();
    }

    pub fn cancel_throw(&mut self) {
        match self.state {
            CharacterMovementState::StartThrowing => self.state = CharacterMovementState::Idle,
            _ => (),
        }

        self.start_animation();
    }

    pub fn throw_done(&mut self) {
        match self.state {
            CharacterMovementState::Throw => self.state = CharacterMovementState::Idle,
            _ => (),
        }

        self.start_animation();
    }

    pub fn hold_throw(&mut self) {
        match self.state {
            CharacterMovementState::StartThrowing => self.state = CharacterMovementState::HoldThrow,
            _ => (),
        }

        self.start_animation();
    }

    pub fn throw(&mut self) {
        match self.state {
            CharacterMovementState::HoldThrow => self.state = CharacterMovementState::Throw,
            _ => (),
        }

        self.start_animation();
    }

    pub fn move_backwards(&mut self) {
        match self.state {
            CharacterMovementState::Idle => self.state = CharacterMovementState::MoveBackwards,
            CharacterMovementState::MoveForwards => {
                self.state = CharacterMovementState::MoveBackwards
            }
            _ => (),
        }

        self.start_animation();
    }

    pub fn stop_move(&mut self) {
        match self.state {
            CharacterMovementState::MoveForwards => self.state = CharacterMovementState::Idle,
            CharacterMovementState::MoveBackwards => self.state = CharacterMovementState::Idle,
            _ => (),
        }

        self.start_animation();
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

    fn start_animation(&mut self) {
        if let Some(animation) = self.animation_states.get_mut(&self.state) {
            self.time_in_current_state_in_ms = 0.0;
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
