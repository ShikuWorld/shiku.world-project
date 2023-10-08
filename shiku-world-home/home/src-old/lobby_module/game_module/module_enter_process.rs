use crate::core::entity::def::EntityId;
use crate::core::module::{GuestEvent, GuestStateChange, ModuleOutputSender, ModuleToSystemEvent};
use crate::resource_module::def::GuestId;
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

type SlotId = String;

pub struct ModuleEnterProcesses {
    pub processes: HashMap<SlotId, ModuleEnterProcess>,
    pub guest_id_to_exit_area_map: HashMap<GuestId, (SlotId, EntityId)>,
    pub guest_being_processed: HashSet<GuestId>,
}

pub struct ModuleEnterProcess {
    pub exit_slot_id: String,
    pub exit_area_guest_map: HashMap<EntityId, HashSet<GuestId>>,
    pub amount_per_area: usize,
    pub opened: bool,
    pub open_on_amount_reached: bool,
    pub enter_cooldown_in_ms: u128,
    pub last_entered: Instant,
    pub info_text: String,
}

impl ModuleEnterProcesses {
    pub fn new() -> ModuleEnterProcesses {
        ModuleEnterProcesses {
            processes: HashMap::new(),
            guest_id_to_exit_area_map: HashMap::new(),
            guest_being_processed: HashSet::new(),
        }
    }

    pub fn is_guest_being_processed(&self, guest_id: &GuestId) -> bool {
        self.guest_being_processed.contains(guest_id)
    }

    pub fn update_enter_processes(&mut self, module_output_sender: &mut ModuleOutputSender) {
        for process in self.processes.values_mut() {
            if process.open_on_amount_reached {
                Self::update_enter_process(
                    module_output_sender,
                    &mut self.guest_being_processed,
                    &mut self.guest_id_to_exit_area_map,
                    process,
                );
            } else {
                Self::update_matchmaking_process(
                    module_output_sender,
                    &mut self.guest_being_processed,
                    &mut self.guest_id_to_exit_area_map,
                    process,
                );
            }
        }
    }

    fn update_matchmaking_process(
        module_output_sender: &mut ModuleOutputSender,
        guest_being_processed: &mut HashSet<GuestId>,
        guest_id_to_exit_area_map: &mut HashMap<GuestId, (SlotId, EntityId)>,
        process: &mut ModuleEnterProcess,
    ) {
        let all_areas_have_enough_guests = process
            .exit_area_guest_map
            .values()
            .all(|guest_hashset| guest_hashset.len() >= process.amount_per_area);
        if all_areas_have_enough_guests
            && Instant::now()
                .duration_since(process.last_entered)
                .as_millis()
                > process.enter_cooldown_in_ms
        {
            for guest_hashset in process.exit_area_guest_map.values_mut() {
                let guest_to_remove: Vec<GuestId> = guest_hashset
                    .clone()
                    .into_iter()
                    .take(process.amount_per_area)
                    .collect();
                for guest_id in guest_to_remove {
                    if guest_hashset.remove(&guest_id) {
                        if let Some((slot_id, _exit_area_id)) =
                            guest_id_to_exit_area_map.remove(&guest_id)
                        {
                            guest_being_processed.insert(guest_id);
                            Self::send_guest_to_module(guest_id, slot_id, module_output_sender);
                        }
                    }
                }
            }

            process.last_entered = Instant::now();
            process.update_info_text();
        }
    }

    fn update_enter_process(
        module_output_sender: &mut ModuleOutputSender,
        guest_being_processed: &mut HashSet<GuestId>,
        guest_id_to_exit_area_map: &mut HashMap<GuestId, (SlotId, EntityId)>,
        process: &mut ModuleEnterProcess,
    ) {
        if process.opened {
            for guest_hashset in process.exit_area_guest_map.values_mut() {
                for guest_id in guest_hashset.drain() {
                    if let Some((slot_id, _exit_area_id)) =
                        guest_id_to_exit_area_map.remove(&guest_id)
                    {
                        guest_being_processed.insert(guest_id);
                        Self::send_guest_to_module(guest_id, slot_id, module_output_sender);
                    }
                }
            }
            return;
        }

        let all_areas_have_enough_guests = process
            .exit_area_guest_map
            .values()
            .all(|guest_hashset| guest_hashset.len() >= process.amount_per_area);
        if all_areas_have_enough_guests {
            process.opened = true;
            process.update_info_text();
        }
    }

    pub fn on_guest_leave(&mut self, guest_id: &GuestId) {
        debug!("On guest leave.");
        self.remove_guest_from_its_area(guest_id);
        self.guest_being_processed.remove(guest_id);
    }

    fn send_guest_to_module(
        guest_id: GuestId,
        slot_id: String,
        module_output_sender: &mut ModuleOutputSender,
    ) {
        if let Err(err) = module_output_sender
            .module_to_system_sender
            .send(GuestEvent {
                guest_id,
                event_type: ModuleToSystemEvent::GuestStateChange(GuestStateChange::ExitModule(
                    slot_id,
                )),
            })
        {
            error!(
                "Could not send exit module event, this is very bad! {:?}",
                err
            );
        }
    }

    pub fn add_enter_process(
        &mut self,
        exit_slot_id: SlotId,
        amount_per_area: usize,
        open_on_amount_reached: bool,
    ) {
        self.processes.insert(
            exit_slot_id.clone(),
            ModuleEnterProcess::new(exit_slot_id, amount_per_area, open_on_amount_reached),
        );
    }

    pub fn add_exit_area(&mut self, slot_id: &SlotId, entity_id: EntityId) {
        if let Some(process) = self.processes.get_mut(slot_id) {
            process
                .exit_area_guest_map
                .insert(entity_id, HashSet::new());
            process.update_info_text();
        }
    }

    pub fn add_guest_to_exit_area(
        &mut self,
        slot_id: &SlotId,
        entity_id: &EntityId,
        guest_id: &GuestId,
    ) {
        if let Some(process) = self.processes.get_mut(slot_id) {
            if let Some(exit_area_hashmap) = process.exit_area_guest_map.get_mut(entity_id) {
                if !exit_area_hashmap.contains(guest_id) {
                    exit_area_hashmap.insert(guest_id.clone());
                    self.guest_id_to_exit_area_map
                        .insert(guest_id.clone(), (slot_id.clone(), entity_id.clone()));
                    process.update_info_text();
                }
            }
        }
    }

    pub fn get_exit_area_guest_count(&mut self, slot_id: &SlotId, entity_id: &EntityId) -> usize {
        if let Some(process) = self.processes.get_mut(slot_id) {
            if let Some(exit_area_hashmap) = process.exit_area_guest_map.get_mut(entity_id) {
                return exit_area_hashmap.len();
            }
        }

        0
    }

    pub fn remove_guest_from_its_area(&mut self, guest_id: &GuestId) {
        if let Some((slot_id, exit_area_id)) = self.guest_id_to_exit_area_map.remove(guest_id) {
            if let Some(process) = self.processes.get_mut(&slot_id) {
                if let Some(guest_hash_set) = process.exit_area_guest_map.get_mut(&exit_area_id) {
                    guest_hash_set.remove(guest_id);
                    process.update_info_text();
                }
            }
        }
    }

    pub fn is_guest_in_exit_area(
        &self,
        slot_id: &SlotId,
        entity_id: &EntityId,
        guest_id: &GuestId,
    ) -> bool {
        if let Some(process) = self.processes.get(slot_id) {
            if let Some(exit_area_hashset) = process.exit_area_guest_map.get(entity_id) {
                return exit_area_hashset.contains(guest_id);
            }
        }

        false
    }
}

impl ModuleEnterProcess {
    pub fn new(
        exit_slot_id: String,
        amount_per_area: usize,
        open_on_amount_reached: bool,
    ) -> ModuleEnterProcess {
        ModuleEnterProcess {
            exit_slot_id,
            amount_per_area,
            opened: false,
            open_on_amount_reached,
            last_entered: Instant::now(),
            enter_cooldown_in_ms: 1000,
            exit_area_guest_map: HashMap::new(),
            info_text: "".into(),
        }
    }

    pub fn update_info_text(&mut self) {
        if self.opened {
            self.info_text = "OPEN".into();
        } else {
            self.info_text = format!(
                "{} |{}",
                self.get_total_guest_count(),
                self.get_total_max_guest_count()
            );
        }
    }

    pub fn get_total_guest_count(&self) -> usize {
        return self
            .exit_area_guest_map
            .iter()
            .map(|(entity_id, guest_hashset)| guest_hashset.len())
            .sum();
    }

    pub fn get_total_max_guest_count(&self) -> usize {
        return self.exit_area_guest_map.len() * self.amount_per_area;
    }
}
