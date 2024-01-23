<template>
  <v-card class="create-tileset" title="Create new Map">
    <v-text-field label="Name" v-model="game_map.name"></v-text-field>
    <v-text-field
      label="Path inside module folder"
      type="number"
      density="comfortable"
      v-model="game_map.resource_path"
    ></v-text-field>
    <v-text-field
      label="Tile width"
      type="number"
      density="comfortable"
      v-model="game_map.tile_width"
    ></v-text-field>
    <v-text-field
      label="Tile height"
      density="comfortable"
      type="number"
      v-model="game_map.tile_height"
    ></v-text-field>
    <v-text-field
      label="Chunk size"
      density="comfortable"
      type="number"
      v-model="game_map.chunk_size"
    ></v-text-field>
    <v-card-actions>
      <v-spacer></v-spacer>

      <v-btn text="Create Map" @click="save_map"></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>
<script lang="ts" setup>
import { GameMap } from "@/client/communication/api/blueprints/GameMap";
import { reactive, toRefs } from "vue";
import { Module } from "@/editor/blueprints/Module";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);
const game_map = reactive<GameMap>({
  world_id: "",
  module_id: module.value.id,
  chunk_size: 8,
  main_scene: {
    id: "root",
    name: "Main",
    resource_path: "",
    root_node: {Group: {
      name: "Main",
      id: "",
      inherits: null,
      data: "",
      children:[]}}
  },
  name: "",
  resource_path: "",
  terrain: {
    BG10: {},
    BG09: {},
    BG08: {},
    BG07: {},
    BG06: {},
    BG05: {},
    BG04: {},
    BG03: {},
    BG02: {},
    BG01: {},
    BG00: {},
    ObjectsBelow: {},
    Terrain: {},
    ObjectsFront: {},
    FG00: {},
    FG01: {},
    FG02: {},
    FG03: {},
    FG04: {},
    FG05: {},
    FG06: {},
    FG07: {},
    FG08: {},
    FG09: {},
    FG10: {},
  },
  tile_height: 16,
  tile_width: 16,
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", game_map: GameMap): void;
}>();
function save_map() {
  game_map.resource_path = `modules/${module.value.name}`;
  emit("save", game_map);
}
</script>
<style>
.create-tileset {
  display: flex;
  flex-direction: column;
}
</style>
