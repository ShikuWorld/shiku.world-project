<template>
  <v-card class="create-tileset" title="Create new Map">
    <v-text-field label="Name" v-model="partial_scene.name"></v-text-field>
    <v-text-field
      label="Path to file"
      density="comfortable"
      v-model="partial_scene.resource_path"
    ></v-text-field>
    <v-select
      label="Root Node Type"
      v-model="selected_root_node_type"
      :items="root_node_types"
    ></v-select>
    <v-card-actions>
      <v-spacer></v-spacer>

      <v-btn text="Create Scene" @click="save_scene"></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>
<script lang="ts" setup>
import { reactive, ref } from "vue";
import { Scene } from "@/editor/blueprints/Scene";
import { v4 as uuidv4 } from "uuid";
import { KeysOfUnion } from "@/editor/utils";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { create_game_node } from "@/editor/stores/editor";

const partial_scene = reactive<Partial<Scene>>({
  id: uuidv4(),
  name: "Scene",
  resource_path: "",
});

const selected_root_node_type = ref<KeysOfUnion<GameNodeKind>>("Node");
const root_node_types: KeysOfUnion<GameNodeKind>[] = [
  "Node",
  "RigidBody",
  "Render",
];

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", scene: Scene): void;
}>();
function save_scene() {
  if (
    !partial_scene.id ||
    !partial_scene.name ||
    !partial_scene.resource_path
  ) {
    console.error("All scene params need to be set!");
    return;
  }
  emit("save", {
    id: partial_scene.id,
    name: partial_scene.name,
    resource_path: partial_scene.resource_path,
    root_node: create_game_node(selected_root_node_type.value),
  });
}
</script>
<style>
.create-tileset {
  display: flex;
  flex-direction: column;
}
</style>
