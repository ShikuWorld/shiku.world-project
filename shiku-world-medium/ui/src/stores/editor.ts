import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { ModuleUpdate } from "@/editor/blueprints/ModuleUpdate";
import { Conductor } from "@/editor/blueprints/Conductor";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";

export interface EditorStore {
  editor_open: boolean;
  main_door_status: boolean;
  selected_module_id: string;
  current_main_instance_id: string;
  edit_module_id: string;
  conductor: Conductor;
  tileset_map: { [tileset_path: string]: Tileset };
  current_map_index: number;
  modules: { [module_id: string]: Module };
}
export const use_editor_store = defineStore("editor", {
  state: (): EditorStore => ({
    editor_open: false,
    modules: {},
    main_door_status: false,
    selected_module_id: "",
    current_main_instance_id: "",
    edit_module_id: "",
    current_map_index: 0,
    tileset_map: {},
    conductor: { module_connection_map: {} },
  }),
  actions: {
    set_selected_module_id(id: string) {
      this.selected_module_id = id;
    },
    set_current_main_instance_id(id: string) {
      this.current_main_instance_id = id;
    },
    set_main_module_to_edit(module_id: string) {
      sendAdminEvent({ SelectMainModuleToEdit: module_id });
    },
    update_module(module: Partial<Module> & { id: string }) {
      if (module.id) {
        this.modules = {
          ...this.modules,
          [module.id]: { ...this.modules[module.id], ...module },
        };
      }
    },
    delete_module(module_id: string) {
      const modules = {
        ...this.modules,
      };
      delete modules[module_id];
      this.modules = modules;
    },
    create_module(module: Module) {
      this.modules = {
        ...this.modules,
        [module.id]: module,
      };
    },
    get_module(id: string) {
      return this.modules[id];
    },
    set_modules(modules: Module[]) {
      this.modules = modules.reduce(
        (current, module) => ({ ...current, [module.id]: module }),
        {},
      );
    },
    set_tileset(tileset: Tileset) {
      this.tileset_map = {
        ...this.tileset_map,
        [tileset_key(tileset)]: tileset,
      };
    },
    delete_tileset(tileset: Tileset) {
      const tileset_map = { ...this.tileset_map };
      delete tileset_map[tileset_key(tileset)];
      this.tileset_map = tileset_map;
    },
    set_main_door_status(status: boolean) {
      this.main_door_status = status;
    },
    load_modules() {
      sendAdminEvent("LoadEditorData");
    },
    set_conductor(conductor: Conductor) {
      this.conductor = conductor;
    },
    save_module_server(
      module_id: string,
      module_update: Partial<ModuleUpdate>,
    ) {
      sendAdminEvent({
        UpdateModule: [
          module_id,
          {
            name: null,
            maps: null,
            exit_points: null,
            max_guests: null,
            min_guests: null,
            resources: null,
            insert_points: null,
            ...module_update,
          },
        ],
      });
    },
    create_tileset_server(tileset: Tileset) {
      sendAdminEvent({ CreateTileset: tileset });
    },
    update_tileset_server(tileset: Tileset) {
      sendAdminEvent({ UpdateTileset: tileset });
    },
    delete_tileset_server(tileset: Tileset) {
      sendAdminEvent({ DeleteTileset: tileset });
    },
    create_module_server(name: string) {
      sendAdminEvent({ CreateModule: name });
    },
    delete_module_server(id: string) {
      sendAdminEvent({ DeleteModule: id });
    },
    save_conductor_server(conductor: Conductor) {
      sendAdminEvent({ UpdateConductor: conductor });
    },
    show_editor() {
      this.editor_open = true;
    },
    hide_editor() {
      this.editor_open = false;
    },
  },
});

function sendAdminEvent(event: AdminToSystemEvent) {
  if (window.medium.communication_state.is_connection_open) {
    window.medium.communication_state.ws_connection.send(JSON.stringify(event));
  }
}

function tileset_key(tileset: Tileset) {
  return `${tileset.resource_path}/${tileset.name}.tileset.json`;
}
