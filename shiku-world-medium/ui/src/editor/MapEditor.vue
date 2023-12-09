<template>
  <div class="map-editor" :style="grid_vars">
    <div
      class="map-editor--grid"
      ref="editor_element"
      @click="emit_tile_click"
      @mousemove="move_selected_tile"
    >
      <div class="map-editor__selected-tile" :style="tile_position_vars"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { GameMap } from "@/editor/blueprints/GameMap";
import { computed, onMounted, ref, toRefs } from "vue";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";
import { LayerKind } from "@/editor/blueprints/LayerKind";

const props = defineProps<{ map: GameMap }>();
const { map } = toRefs(props);
const editor_element = ref<HTMLElement>();
const layer = ref<LayerKind>("Terrain");
onMounted(() => {
  camera.value.set_camera_zoom(1.5);
});

const emit = defineEmits<{
  (e: "tile_click", layer: LayerKind, tile_x: number, tile_y: number): void;
}>();

const tile_width = computed(() => map.value.tile_width * camera.value.zoom);
const tile_height = computed(() => map.value.tile_height * camera.value.zoom);

// const { browse_folder, toggle_resource_on_module } = use_editor_store();
const grid_vars = computed(() => ({
  "--tile-width": `${tile_width.value * camera.value.zoom}px`,
  "--tile-height": `${tile_height.value * camera.value.zoom}px`,
}));
const tile_position_vars = computed(() => ({
  "--selected-tile-pos-x": `${selected_tile_position.value.x}px`,
  "--selected-tile-pos-y": `${selected_tile_position.value.y}px`,
}));
const selected_tile_position = ref({
  x: 0,
  y: 0,
});
const { camera } = storeToRefs(use_editor_store());

function emit_tile_click($event: MouseEvent) {
  if (!editor_element.value) {
    return;
  }
  let bounding_rect = editor_element.value.getBoundingClientRect();
  let tile_x = Math.round(
    ($event.pageX -
      bounding_rect.left -
      (tile_width.value / 2) * camera.value.zoom) /
      (tile_width.value * camera.value.zoom),
  );
  let tile_y = Math.round(
    ($event.pageY -
      bounding_rect.top -
      (tile_height.value / 2) * camera.value.zoom) /
      (tile_height.value * camera.value.zoom),
  );
  emit("tile_click", layer.value, tile_x, tile_y);
}
function move_selected_tile($event: MouseEvent) {
  if (!editor_element.value) {
    return;
  }
  let bounding_rect = editor_element.value.getBoundingClientRect();
  let tile_x =
    Math.round(
      ($event.pageX -
        bounding_rect.left -
        (tile_width.value / 2) * camera.value.zoom) /
        (tile_width.value * camera.value.zoom),
    ) *
    tile_width.value *
    camera.value.zoom;
  let tile_y =
    Math.round(
      ($event.pageY -
        bounding_rect.top -
        (tile_height.value / 2) * camera.value.zoom) /
        (tile_height.value * camera.value.zoom),
    ) *
    tile_height.value *
    camera.value.zoom;
  if (
    selected_tile_position.value.x != tile_x ||
    selected_tile_position.value.y != tile_y
  ) {
    selected_tile_position.value = { x: tile_x, y: tile_y };
  }
}
</script>

<style>
.map-editor {
  --tile-width: 8px;
  --tile-height: 8px;
  --map-offset-x: 0px;
  --map-offset-y: 0px;
  --selected-tile-pos-x: 0px;
  --selected-tile-pos-y: 0px;
}
.map-editor__selected-tile {
  left: var(--selected-tile-pos-x);
  top: var(--selected-tile-pos-y);
  width: var(--tile-width);
  height: var(--tile-height);
  border: 1px solid blue;
  position: absolute;
}
.map-editor--grid {
  background-size: var(--tile-width) var(--tile-height);
  background-image: linear-gradient(
      to right,
      rgba(255, 255, 255, 0.33) 1px,
      transparent 1px
    ),
    linear-gradient(to bottom, rgba(255, 255, 255, 0.33) 1px, transparent 1px);
  background-repeat: repeat;
  display: inline-block;
  width: 100%;
  height: 100%;
}
</style>
