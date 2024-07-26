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
  </div>
</template>

<script lang="ts" setup>
import { GameMap } from "@/editor/blueprints/GameMap";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { toRefs } from "vue";
import { use_resources_store } from "@/editor/stores/resources";

const props = defineProps<{
  game_map: GameMap;
}>();
const { game_map } = toRefs(props);
const { update_map_server } = use_resources_store();

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
</script>
<style></style>
