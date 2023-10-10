import { ref } from "vue";
import { defineStore } from "pinia";
import type { Entity } from "@/editor/blueprints/Entity";
import type { Module } from "@/editor/blueprints/Module";

export const useEditorStore = defineStore("editor", () => {
  const currentModule: Module = {
    name: "undefined",
    maps: [],
    max_guests: 0,
    min_guests: 0,
    resources: [],
    insert_points: [],
    exit_points: [],
  };
  const realEntityToBlueprintMap = ref(new Map<string, Entity>());

  return { currentModule, realEntityToBlueprintMap };
});
