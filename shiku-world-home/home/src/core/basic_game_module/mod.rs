use crate::core::blueprint::def::CameraSettings;
use crate::core::rapier_simulation::def::RapierSimulation;
use log::error;
use rapier2d::prelude::{Real, Vector};

use crate::core::entity_manager::EntityManager;
use crate::core::get_out_dir;
use crate::resource_module::map::def::TiledMap;

pub struct BasicGameModule<T: EntityManager, S> {
    pub simulation: RapierSimulation,
    pub tiled_map: Option<TiledMap>,
    pub game_entity_manager: T,
    pub simulation_config: S,
    pub base_camera_settings: Option<CameraSettings>,
}

impl<T: EntityManager, S> BasicGameModule<T, S> {
    pub fn new(
        mut game_entity_manager: T,
        simulation_config: S,
        map_file_path: &str,
    ) -> BasicGameModule<T, S> {
        let mut simulation = RapierSimulation::new();

        let tiled_map = match TiledMap::from_xml(get_out_dir(), map_file_path) {
            Ok(tiled_map) => {
                game_entity_manager.create_initial(&tiled_map, &mut simulation);

                Some(tiled_map)
            }
            Err(err) => {
                error!("{:?}", err);

                None
            }
        };

        BasicGameModule {
            simulation,
            tiled_map,
            game_entity_manager,
            simulation_config,
            base_camera_settings: None,
        }
    }

    pub fn set_gravity(&mut self, gravity: Vector<Real>) {
        self.simulation.set_gravity(gravity);
    }

    pub fn update(&mut self) {
        self.simulation.update();
        self.game_entity_manager
            .update_entity_positions(&mut self.simulation);
    }
}
