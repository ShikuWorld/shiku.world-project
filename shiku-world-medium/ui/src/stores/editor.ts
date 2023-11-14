import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { ModuleUpdate } from "@/editor/blueprints/ModuleUpdate";
import { Conductor } from "@/editor/blueprints/Conductor";

export interface EditorStore {
  editor_open: boolean;
  main_door_status: boolean;
  selected_module_id: string;
  conductor: Conductor;
  modules: { [module_id: string]: Module };
}
export const use_editor_store = defineStore("editor", {
  state: (): EditorStore => ({
    editor_open: false,
    modules: {},
    main_door_status: false,
    selected_module_id: "",
    conductor: { module_connection_map: {} },
  }),
  actions: {
    set_selected_module_id(id: string) {
      this.selected_module_id = id;
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
