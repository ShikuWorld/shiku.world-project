use crate::core::blueprint::def::{BlueprintResource, BlueprintService, Conductor, ResourceKind};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::module::EditorEvent;
use crate::resource_module::def::LoadResource;
use log::error;
use rapier2d::prelude::Real;
use std::path::PathBuf;

pub fn save_and_send_conductor_update<F>(conductor: Conductor, send_editor_event: &mut F)
where
    F: FnMut(EditorEvent),
{
    match BlueprintService::save_conductor_blueprint(&conductor) {
        Ok(()) => {
            send_editor_event(EditorEvent::UpdatedConductor(conductor));
        }
        Err(err) => {
            error!("Could not save conductor {:?}", err)
        }
    }
}

pub fn loading_resources_from_blueprint_resource(
    blueprint_resource: &BlueprintResource,
) -> Vec<LoadResource> {
    match blueprint_resource.kind {
        ResourceKind::Tileset => {
            match Blueprint::load_tileset(PathBuf::from(blueprint_resource.path.clone())) {
                Ok(tileset) => tileset
                    .get_image_paths()
                    .iter()
                    .map(|path| LoadResource::image(path.clone()))
                    .collect(),
                Err(err) => {
                    error!("Could not load tileset! {:?}", err);
                    Vec::new()
                }
            }
        }
        ResourceKind::Font => {
            match Blueprint::load_font(PathBuf::from(blueprint_resource.path.clone())) {
                Ok(font) => vec![LoadResource::font(font.font_path.clone())],
                Err(err) => {
                    error!("Could not load font! {:?}", err);
                    Vec::new()
                }
            }
        }
        ResourceKind::Audio => {
            match Blueprint::load_audio(PathBuf::from(blueprint_resource.path.clone())) {
                Ok(font) => vec![LoadResource::audio(font.resource_path.clone())],
                Err(err) => {
                    error!("Could not load audio! {:?}", err);
                    Vec::new()
                }
            }
        }
        ResourceKind::Scene
        | ResourceKind::Map
        | ResourceKind::Unknown
        | ResourceKind::Script
        | ResourceKind::CharacterAnimation => Vec::new(),
    }
}

pub fn bring_polygon_in_clockwise_order(vertices: &mut [(Real, Real)]) {
    // Find the centroid of the polygon
    let mut centroid_x = 0.0;
    let mut centroid_y = 0.0;
    for &(x, y) in vertices.iter() {
        centroid_x += x;
        centroid_y += y;
    }
    let n = vertices.len() as Real;
    centroid_x /= n;
    centroid_y /= n;

    // Sort the vertices based on their angle with respect to the centroid
    vertices.sort_by(|&(x1, y1), &(x2, y2)| {
        let dx1 = x1 - centroid_x;
        let dy1 = y1 - centroid_y;
        let dx2 = x2 - centroid_x;
        let dy2 = y2 - centroid_y;

        let cross_product = dx1 * dy2 - dx2 * dy1;
        if cross_product > 0.0 {
            std::cmp::Ordering::Less
        } else if cross_product < 0.0 {
            std::cmp::Ordering::Greater
        } else {
            // If the cross product is zero, the points are collinear.
            // Compare the distances from the centroid to determine the order.
            let dist1 = dx1 * dx1 + dy1 * dy1;
            let dist2 = dx2 * dx2 + dy2 * dy2;
            dist1.partial_cmp(&dist2).unwrap()
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_polygon_vertices() {
        let mut vertices1 = vec![(0.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
        bring_polygon_in_clockwise_order(&mut vertices1);
        assert_eq!(
            vertices1,
            vec![(-1.0, 0.0), (0.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        );

        let mut vertices2 = vec![(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
        bring_polygon_in_clockwise_order(&mut vertices2);
        assert_eq!(
            vertices2,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        );
    }
}
