import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";

export interface EditorStore {
  editor_open: boolean;
  main_door_status: boolean;
  selected_module_id: string;
  modules: { [module_id: string]: Module };
}
export const use_editor_store = defineStore("editor", {
  state: (): EditorStore => ({
    editor_open: false,
    modules: {},
    main_door_status: false,
    selected_module_id: "",
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
      if (window.medium.communication_state.is_connection_open) {
        window.medium.communication_state.ws_connection.send(
          JSON.stringify("LoadEditorData" as AdminToSystemEvent),
        );
      }
    },
    show_editor() {
      this.editor_open = true;
    },
    hide_editor() {
      this.editor_open = false;
    },
  },
});
