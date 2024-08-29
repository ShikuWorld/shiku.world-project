<template>
  <v-card class="create-tileset" title="Create new Map">
    <v-text-field label="Name" v-model="game_map.name"></v-text-field>
    <v-text-field
      label="Path inside module folder"
      density="comfortable"
      v-model="game_map.resource_path"
    ></v-text-field>
    <v-number-input
      label="Tile width"
      density="comfortable"
      :model-value="game_map.tile_width"
      @update:model-value="(width) => (game_map.tile_width = width)"
    ></v-number-input>
    <v-number-input
      label="Tile height"
      density="comfortable"
      :model-value="game_map.tile_height"
      @update:model-value="(height) => (game_map.tile_height = height)"
    ></v-number-input>
    <v-number-input
      label="Chunk size"
      density="comfortable"
      type="number"
      v-model="game_map.chunk_size"
    ></v-number-input>
    <v-select
      label="Root node"
      v-model="root_node_selection"
      :items="root_node_options"
    ></v-select>
    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn text="Create Map" @click="save_map"></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>
<script lang="ts" setup>
import { GameMap } from "@/client/communication/api/blueprints/GameMap";
import { reactive, toRefs, ref, computed } from "vue";
import { Module } from "@/editor/blueprints/Module";
import { VNumberInput } from "vuetify/labs/VNumberInput";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);
const root_node_selection = ref("");
const root_node_options = computed(() =>
  module.value.resources.filter((r) => r.kind === "Scene").map((r) => r.path),
);
const game_map = reactive<GameMap>({
  world_id: "",
  module_id: module.value.id,
  chunk_size: 8,
  main_scene: "",
  name: "",
  resource_path: "",
  physics_settings: {
    gravity: [0, 9.81],
  },
  layer_parallax: {
    BG10: [1.0, 1.0],
    BG09: [1.0, 1.0],
    BG08: [1.0, 1.0],
    BG07: [1.0, 1.0],
    BG06: [1.0, 1.0],
    BG05: [1.0, 1.0],
    BG04: [1.0, 1.0],
    BG03: [1.0, 1.0],
    BG02: [1.0, 1.0],
    BG01: [1.0, 1.0],
    BG00: [1.0, 1.0],
    ObjectsBelow: [1.0, 1.0],
    Terrain: [1.0, 1.0],
    ObjectsFront: [1.0, 1.0],
    FG00: [1.0, 1.0],
    FG01: [1.0, 1.0],
    FG02: [1.0, 1.0],
    FG03: [1.0, 1.0],
    FG04: [1.0, 1.0],
    FG05: [1.0, 1.0],
    FG06: [1.0, 1.0],
    FG07: [1.0, 1.0],
    FG08: [1.0, 1.0],
    FG09: [1.0, 1.0],
    FG10: [1.0, 1.0],
  },
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
  camera_settings: {
    zoom: 1.0,
    bounds: null,
  },
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", game_map: GameMap): void;
}>();
function save_map() {
  game_map.resource_path = `modules/${module.value.name}`;
  game_map.main_scene = root_node_selection.value;
  emit("save", game_map);
}
</script>
<style>
.create-tileset {
  display: flex;
  flex-direction: column;
}
</style>
