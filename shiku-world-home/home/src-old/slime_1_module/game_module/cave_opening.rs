use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity::def::EntityId;
use crate::core::entity::render::{ShakeScreenEffect, ShowEffect};
use crate::slime_1_module::def::Slime1BasicGameModule;
use crate::slime_1_module::def::Slime1SimulationConfig;
use crate::slime_1_module::game_module::generated::{
    Debris, DebrisEntity, Slime1GameEntityManager,
};
use log::debug;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use rapier2d::prelude::{Real, Vector};
use std::collections::HashMap;

pub struct CaveOpening {
    started: bool,
    done: bool,
    current_time: Real,
    current_y_water: Real,
    debris_chance: Vec<(String, Real, Real, Real)>,
    debris_emitter: Vec<(Real, Real)>,
    rng: ThreadRng,
}

impl CaveOpening {
    pub fn is_running(&self) -> bool {
        self.started && !self.done
    }
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub fn new(basic_game_module: &mut Slime1BasicGameModule) -> CaveOpening {
        let mut debris_emitter = Vec::new();
        for debris_id in basic_game_module
            .game_entity_manager
            .debris_map
            .values()
            .map(|d| d.id.clone())
            .collect::<Vec<EntityId>>()
        {
            if let Some(debris_entity) = basic_game_module
                .game_entity_manager
                .remove_debris(&debris_id, &mut basic_game_module.simulation)
            {
                debris_emitter.push((
                    debris_entity.isometry.translation.x,
                    debris_entity.isometry.translation.y,
                ));
            }
        }

        let overall_chance = 2.0;

        let debris_chance = vec![
            (
                Debris::VARIANTS.default.gid_formation_1.to_string(),
                overall_chance / 500.0,
                4000.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_formation_2.to_string(),
                overall_chance / 500.0,
                4000.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_formation_3.to_string(),
                overall_chance / 500.0,
                4000.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_formation_4.to_string(),
                overall_chance / 500.0,
                4000.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_formation_5.to_string(),
                overall_chance / 500.0,
                4000.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_l_1.to_string(),
                overall_chance / 250.0,
                200.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_l_2.to_string(),
                overall_chance / 250.0,
                200.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_l_3.to_string(),
                overall_chance / 250.0,
                200.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_l_4.to_string(),
                overall_chance / 250.0,
                200.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_m_1.to_string(),
                overall_chance / 100.0,
                100.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_m_2.to_string(),
                overall_chance / 100.0,
                100.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_m_3.to_string(),
                overall_chance / 100.0,
                100.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_m_4.to_string(),
                overall_chance / 100.0,
                100.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_m_5.to_string(),
                overall_chance / 100.0,
                100.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_m_6.to_string(),
                overall_chance / 100.0,
                100.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_s_1.to_string(),
                overall_chance / 50.0,
                20.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_s_2.to_string(),
                overall_chance / 50.0,
                20.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_s_3.to_string(),
                overall_chance / 50.0,
                20.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_s_4.to_string(),
                overall_chance / 50.0,
                20.0,
                0.0,
            ),
            (
                Debris::VARIANTS.default.gid_stone_s_5.to_string(),
                overall_chance / 50.0,
                20.0,
                0.0,
            ),
        ];

        CaveOpening {
            started: false,
            done: false,
            current_time: 0.0,
            current_y_water: 14.0,
            debris_emitter,
            debris_chance,
            rng: thread_rng(),
        }
    }

    pub fn open(
        &mut self,
        cave_wall_door: &EntityId,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        self.started = true;

        basic_game_module
            .game_entity_manager
            .remove_door(cave_wall_door, &mut basic_game_module.simulation);

        basic_game_module
            .game_entity_manager
            .new_show_effects
            .push(ShowEffect::ShakeScreenEffect(ShakeScreenEffect {
                is_bidirectional: true,
                shake_amount: 8,
                shake_count_max: 250,
                shake_delay: 40,
            }));
    }

    pub fn update(&mut self, time: Real, basic_game_module: &mut Slime1BasicGameModule) {
        if !self.started || self.done {
            return;
        }

        self.current_time += time;
        self.current_y_water += 16.0 / basic_game_module.simulation.simulation_scaling_factor;

        let water_to_remove_ids: Vec<String> = basic_game_module
            .game_entity_manager
            .great_waterfall_tiles_map
            .values()
            .filter(|f| f.isometry.translation.y < self.current_y_water)
            .map(|f| f.id.clone())
            .collect();

        for water_to_remove_id in water_to_remove_ids {
            basic_game_module
                .game_entity_manager
                .remove_great_waterfall_tiles(
                    &water_to_remove_id,
                    &mut basic_game_module.simulation,
                );
        }

        if self.current_time < 7500.0 {
            for (_id, _chance, delay, last_used) in self.debris_chance.iter_mut() {
                if *last_used > *delay {
                    *last_used = 0.0;
                }

                if *last_used > 0.0 {
                    *last_used += time;
                }
            }

            for (x, y) in self.debris_emitter.iter() {
                let rng_throw: Real = self.rng.gen();

                for (id, chance, _delay, last_used) in self.debris_chance.iter_mut() {
                    if *chance > rng_throw && *last_used == 0.0 {
                        *last_used = time;
                        let x_rand = (self.rng.gen::<Real>() - 0.5) * 32.0;
                        let y_rand = (self.rng.gen::<Real>() - 1.0) * 32.0;
                        basic_game_module.game_entity_manager.create_debris(
                            Debris {},
                            Vector::new(
                                (*x * basic_game_module.simulation.simulation_scaling_factor)
                                    + x_rand,
                                (*y * basic_game_module.simulation.simulation_scaling_factor)
                                    + y_rand,
                            ),
                            &Debris::VARIANTS.default,
                            id.clone(),
                            &mut basic_game_module.simulation,
                            |_d| {},
                        );
                        break;
                    }
                }
            }
        }

        for debris in basic_game_module
            .game_entity_manager
            .debris_map
            .values_mut()
        {
            debris.physics.isometry.translation.y +=
                8.0 / basic_game_module.simulation.simulation_scaling_factor;
        }

        let debris_to_remove_ids: Vec<String> = basic_game_module
            .game_entity_manager
            .debris_map
            .values()
            .filter(|f| f.isometry.translation.y > 33.0)
            .map(|f| f.id.clone())
            .collect();

        for debris_to_remove_id in debris_to_remove_ids {
            basic_game_module
                .game_entity_manager
                .remove_debris(&debris_to_remove_id, &mut basic_game_module.simulation);
        }

        if self.current_time > 10000.0 {
            self.done = true;
        }
    }
}
