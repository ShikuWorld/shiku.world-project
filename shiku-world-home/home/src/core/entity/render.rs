use crate::core::entity::def::EntityId;
use crate::resource_module::map::def::{GeneralObject, LayerName};
use rapier2d::prelude::Real;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum EntityRenderData {
    StaticImage(StaticImage),
    RenderTypeTimer(RenderTypeTimer),
    RenderTypeText(RenderTypeText),
    NoRender(NoRender),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct SimpleImageEffect {
    pub initial_isometrics_2d: (Real, Real, Real),
    pub graphic_id: String,
    pub blending_mode: Option<String>,
    pub transparency: f32,
    pub layer: LayerName,
    pub parent_entity: Option<EntityId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct ShakeScreenEffect {
    pub shake_amount: u32,
    pub shake_delay: u32,
    pub shake_count_max: u32,
    pub is_bidirectional: bool,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum ShowEffect {
    SimpleImageEffect(SimpleImageEffect),
    ShakeScreenEffect(ShakeScreenEffect),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct CameraSettings {
    pub(crate) zoom: Option<Real>,
    pub(crate) bounds: Option<((Real, Real), (Real, Real))>,
}

impl CameraSettings {
    pub fn default() -> CameraSettings {
        CameraSettings {
            bounds: None,
            zoom: Some(1.0),
        }
    }
}

pub trait Renderable {
    fn get_entity_render_data(&self) -> EntityRenderData;
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct NoRender {}

impl Renderable for NoRender {
    fn get_entity_render_data(&self) -> EntityRenderData {
        return EntityRenderData::NoRender(self.clone());
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct StaticImage {
    pub graphic_id: String,
    pub layer: LayerName,
    pub offset_2d: (Real, Real),
    pub scale: (Real, Real),
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tiled: bool,
    pub blending_mode: Option<String>,
}

impl Renderable for StaticImage {
    fn get_entity_render_data(&self) -> EntityRenderData {
        return EntityRenderData::StaticImage(self.clone());
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct RenderTypeTimer {
    pub date: String,
    pub font_family: String,
    pub color: String,
    pub layer: LayerName,
}

impl Renderable for RenderTypeTimer {
    fn get_entity_render_data(&self) -> EntityRenderData {
        return EntityRenderData::RenderTypeTimer(self.clone());
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct RenderTypeText {
    pub text: String,
    pub font_family: String,
    pub color: String,
    pub layer: LayerName,
    pub center_x: bool,
}

impl Renderable for RenderTypeText {
    fn get_entity_render_data(&self) -> EntityRenderData {
        return EntityRenderData::RenderTypeText(self.clone());
    }
}

impl RenderTypeText {
    pub fn from_general_object(
        general_object_option: &Option<GeneralObject>,
        layer_name: LayerName,
    ) -> RenderTypeText {
        if let Some(general_object) = general_object_option {
            if let Some(text) = &general_object.text {
                return RenderTypeText {
                    color: text.color.clone(),
                    text: text.text.clone(),
                    font_family: text.font_family.clone(),
                    layer: layer_name,
                    center_x: false,
                };
            }
        }

        RenderTypeText {
            color: "#000000".to_string(),
            text: "".to_string(),
            font_family: "".to_string(),
            layer: layer_name,
            center_x: false,
        }
    }
}

impl RenderTypeTimer {
    pub fn from_general_object(
        general_object_option: &Option<GeneralObject>,
        layer_name: LayerName,
    ) -> RenderTypeTimer {
        if let Some(general_object) = general_object_option {
            if let Some(text) = &general_object.text {
                return RenderTypeTimer {
                    color: text.color.clone(),
                    date: text.text.clone(),
                    font_family: text.font_family.clone(),
                    layer: layer_name,
                };
            }
        }

        RenderTypeTimer {
            color: "#000000".to_string(),
            date: "".to_string(),
            font_family: "".to_string(),
            layer: layer_name,
        }
    }
}
