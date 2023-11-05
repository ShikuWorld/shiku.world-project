import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";

export interface EditorStore {
  editor_open: boolean;
  main_door_status: boolean;
  modules: Module[];
  current_module: Module;
}
export const use_editor_store = defineStore("editor", {
  state: (): EditorStore => ({
    editor_open: false,
    modules: [],
    main_door_status: false,
    current_module: {
      name: "undefined",
      maps: [],
      max_guests: 0,
      min_guests: 0,
      resources: [],
      insert_points: [],
      exit_points: [],
    },
  }),
  actions: {
    set_modules(modules: Module[]) {
      this.modules = modules;
    },
    set_main_door_status(status: boolean) {
      this.main_door_status = status;
    },
    load_modules() {
      if (window.medium.communication_state.is_connection_open) {
        console.log("sending...?");
        window.medium.communication_state.ws_connection.send(
          JSON.stringify("LoadEditorData" as AdminToSystemEvent),
        );
      }
    },
    show_editor() {
      console.log("shownig editor?");
      this.editor_open = true;
    },
    hide_editor() {
      this.editor_open = false;
    },
  },
});
