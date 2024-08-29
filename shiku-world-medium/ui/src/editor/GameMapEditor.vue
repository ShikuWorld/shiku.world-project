<template>
  <div class="game-map-editor">
    <v-number-input
      :reverse="false"
      label="zoom"
      controlVariant="stacked"
      density="compact"
      :hide-details="true"
      :step="0.1"
      :model-value="game_map.camera_settings.zoom"
      @update:model-value="update_camera_zoom"
    ></v-number-input>
    <v-number-input
      :reverse="false"
      label="gravity x"
      controlVariant="stacked"
      density="compact"
      :hide-details="true"
      :step="0.1"
      :model-value="game_map.physics_settings.gravity[0]"
      @update:model-value="
        (new_value) =>
          update_physics_gravity(
            new_value,
            game_map.physics_settings.gravity[1],
          )
      "
    ></v-number-input>
    <v-number-input
      :reverse="false"
      label="gravity y"
      controlVariant="stacked"
      density="compact"
      :hide-details="true"
      :step="0.1"
      :model-value="game_map.physics_settings.gravity[1]"
      @update:model-value="
        (new_value) =>
          update_physics_gravity(
            game_map.physics_settings.gravity[0],
            new_value,
          )
      "
    ></v-number-input>
    <v-switch
      label="Bounds"
      :model-value="bounds_enabled"
      @update:model-value="toggle_bounds"
    ></v-switch>
    <div v-if="bounds_enabled">
      <v-number-input
        :reverse="false"
        label="x1"
        controlVariant="stacked"
        density="compact"
        :hide-details="true"
        :step="0.1"
        v-if="game_map.camera_settings.bounds"
        :model-value="game_map.camera_settings.bounds[0][0]"
        @update:model-value="
          (new_value) =>
            update_camera_bounds(
              new_value,
              game_map.camera_settings.bounds![0][1],
              game_map.camera_settings.bounds![1][0],
              game_map.camera_settings.bounds![1][1],
            )
        "
      ></v-number-input>
      <v-number-input
        :reverse="false"
        label="y1"
        controlVariant="stacked"
        density="compact"
        :hide-details="true"
        :step="0.1"
        v-if="game_map.camera_settings.bounds"
        :model-value="game_map.camera_settings.bounds[0][1]"
        @update:model-value="
          (new_value) =>
            update_camera_bounds(
              game_map.camera_settings.bounds![0][0],
              new_value,
              game_map.camera_settings.bounds![1][0],
              game_map.camera_settings.bounds![1][1],
            )
        "
      ></v-number-input>
      <v-number-input
        :reverse="false"
        label="x2"
        controlVariant="stacked"
        density="compact"
        :hide-details="true"
        :step="0.1"
        v-if="game_map.camera_settings.bounds"
        :model-value="game_map.camera_settings.bounds[1][0]"
        @update:model-value="
          (new_value) =>
            update_camera_bounds(
              game_map.camera_settings.bounds![0][0],
              game_map.camera_settings.bounds![0][1],
              new_value,
              game_map.camera_settings.bounds![1][1],
            )
        "
      ></v-number-input>
      <v-number-input
        :reverse="false"
        label="y2"
        controlVariant="stacked"
        density="compact"
        :hide-details="true"
        :step="0.1"
        v-if="game_map.camera_settings.bounds"
        :model-value="game_map.camera_settings.bounds[1][1]"
        @update:model-value="
          (new_value) =>
            update_camera_bounds(
              game_map.camera_settings.bounds![0][0],
              game_map.camera_settings.bounds![0][1],
              game_map.camera_settings.bounds![1][0],
              new_value,
            )
        "
      ></v-number-input>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { GameMap } from "@/editor/blueprints/GameMap";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { computed, toRefs } from "vue";
import { use_resources_store } from "@/editor/stores/resources";

const props = defineProps<{
  game_map: GameMap;
}>();
const { game_map } = toRefs(props);
const { update_map_server } = use_resources_store();

function update_physics_gravity(x: number, y: number) {
  update_map_server({
    name: game_map.value.name,
    resource_path: game_map.value.resource_path,
    layer_parallax: null,
    scene: null,
    camera_settings: null,
    physics_settings: {
      ...game_map.value.physics_settings,
      gravity: [x, y],
    },
  });
}

const bounds_enabled = computed(() => !!game_map.value.camera_settings.bounds);
function toggle_bounds(enabled: boolean | null) {
  if (enabled) {
    game_map.value.camera_settings.bounds = [
      [0, 0],
      [1, 1],
    ];
  } else {
    game_map.value.camera_settings.bounds = null;
  }
  update_map_server({
    name: game_map.value.name,
    resource_path: game_map.value.resource_path,
    layer_parallax: null,
    scene: null,
    camera_settings: {
      ...game_map.value.camera_settings,
      bounds: game_map.value.camera_settings.bounds,
    },
  });
}

const update_camera_zoom = (new_value: number) => {
  game_map.value.camera_settings.zoom = new_value;
  update_map_server({
    name: game_map.value.name,
    resource_path: game_map.value.resource_path,
    layer_parallax: null,
    scene: null,
    camera_settings: {
      ...game_map.value.camera_settings,
      zoom: new_value,
    },
  });
};

const update_camera_bounds = (
  x1: number,
  y1: number,
  x2: number,
  y2: number,
) => {
  const bounds: [[number, number], [number, number]] | null = [
    [x1, y1],
    [x2, y2],
  ];
  game_map.value.camera_settings.bounds = bounds;
  update_map_server({
    name: game_map.value.name,
    resource_path: game_map.value.resource_path,
    layer_parallax: null,
    scene: null,
    camera_settings: {
      ...game_map.value.camera_settings,
      bounds,
    },
  });
};
</script>
<style></style>
