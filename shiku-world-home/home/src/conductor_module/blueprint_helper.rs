use crate::core::blueprint::def::{BlueprintService, Conductor};
use crate::core::module::EditorEvent;
use log::error;

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
