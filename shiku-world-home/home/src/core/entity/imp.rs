use crate::core::entity::def::{Entity, EntityId, RemoveEntity, ShowEntity, UpdateEntity};
use crate::core::entity::physics::Physical;
use crate::core::entity::render::{RenderTypeText, Renderable, StaticImage};
use crate::core::rapier_simulation::def::RapierSimulation;
use rapier2d::math::Real;

impl<T, P: Physical, R: Renderable> Entity<T, P, R> {
    pub fn remove_entity(&self) -> RemoveEntity {
        RemoveEntity {
            id: self.id.clone(),
            parent_entity: self.parent_entity.clone(),
        }
    }

    pub fn show_entity(&self) -> ShowEntity {
        ShowEntity {
            id: self.id.clone(),
            parent_entity: self.parent_entity.clone(),
            initial_isometrics_2d: (
                self.isometry.translation.x,
                self.isometry.translation.y,
                self.isometry.rotation.angle(),
            ),
            render: self.render.get_entity_render_data(),
        }
    }

    pub fn update_entity(&self) -> UpdateEntity {
        UpdateEntity {
            id: self.id.clone(),
            render: self.render.get_entity_render_data(),
        }
    }

    pub fn position_update(&self) -> (EntityId, Real, Real, Real) {
        (
            self.id.clone(),
            self.isometry.translation.x,
            self.isometry.translation.y,
            self.isometry.rotation.angle(),
        )
    }

    pub fn update_isometry(&mut self, physics: &RapierSimulation) {
        let current_position = self.physics.position(physics);
        if self.isometry != current_position {
            self.isometry = current_position;
            self.is_position_dirty = true;
        }
    }
}

impl<T, P: Physical> Entity<T, P, StaticImage> {
    pub fn set_graphic_id(&mut self, graphic_id: &'static str) {
        if self.render.graphic_id != graphic_id {
            self.render.graphic_id = String::from(graphic_id);
            self.is_render_dirty = true;
        }
    }

    pub fn set_width(&mut self, width: u32) {
        if let Some(current_width) = self.render.width {
            if current_width != width {
                self.render.width = Some(width);
                self.is_render_dirty = true;
            }
        } else {
            self.render.width = Some(width);
            self.is_render_dirty = true;
        }
    }

    pub fn set_tiled(&mut self, tiled: bool) {
        if self.render.tiled != tiled {
            self.render.tiled = tiled;
            self.is_render_dirty = true;
        }
    }
}

impl<T, P: Physical> Entity<T, P, RenderTypeText> {
    pub fn set_text(&mut self, text: &String) {
        if &self.render.text != text {
            self.render.text = text.clone();
            self.is_render_dirty = true;
        }
    }
}
