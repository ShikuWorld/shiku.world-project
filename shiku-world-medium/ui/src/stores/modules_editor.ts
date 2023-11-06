import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";

export interface ModulesEditorStore {
  selected_module: Module | undefined;
}
export const use_modules_editor_store = defineStore("modules_editor", {
  state: (): ModulesEditorStore => ({
    selected_module: undefined,
  }),
  actions: {
    select_module(module: Module) {
      this.selected_module = module;
    },
  },
});
