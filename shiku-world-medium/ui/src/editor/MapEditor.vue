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
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { use_editor_store } from "@/editor/stores/editor";
import { Isometry } from "@/client/entities";

const { set_camera_position, set_camera_zoom } = use_editor_store();
const props = defineProps<{ instance_id: string; map: GameMap }>();
const { instance_id, map } = toRefs(props);
const editor_element = ref<HTMLElement>();
const camera_zoom = ref(1);
const camera_manual_pos = ref<Isometry>({ x: 0, y: 0, rotation: 0 });
const layer = ref<LayerKind>("Terrain");
onMounted(() => {
  set_camera_zoom(instance_id.value, map.value.world_id, 1.0);
  camera_zoom.value = 1.0;
  setInterval(() => {
    set_camera_position(
      instance_id.value,
      map.value.world_id,
      camera_manual_pos.value,
    );
    camera_manual_pos.value.x += 1;
  }, 500);
});

const emit = defineEmits<{
  (e: "tile_click", layer: LayerKind, tile_x: number, tile_y: number): void;
}>();

const tile_width = computed(() => map.value.tile_width * camera_zoom.value);
const tile_height = computed(() => map.value.tile_height * camera_zoom.value);

const grid_vars = computed(() => ({
  "--map-offset-x": `${
    -(camera_manual_pos.value.x % tile_width.value) * camera_zoom.value
  }px`,
  "--map-offset-y": `${
    -(camera_manual_pos.value.y % tile_height.value) * camera_zoom.value
  }px`,
  "--tile-width": `${tile_width.value * camera_zoom.value}px`,
  "--tile-height": `${tile_height.value * camera_zoom.value}px`,
}));
const tile_position_vars = computed(() => ({
  "--selected-tile-pos-x": `${selected_tile_position.value.x}px`,
  "--selected-tile-pos-y": `${selected_tile_position.value.y}px`,
}));
const selected_tile_position = ref({
  x: 0,
  y: 0,
});

function emit_tile_click($event: MouseEvent) {
  if (!editor_element.value) {
    return;
  }
  let tile_x = Math.round(
    ($event.pageX + camera_manual_pos.value.x) / tile_width.value,
  );
  let tile_y = Math.round(
    ($event.pageY + camera_manual_pos.value.x) / tile_height.value,
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
      ($event.pageX - bounding_rect.left) /
        (tile_width.value * camera_zoom.value),
    ) *
    tile_width.value *
    camera_zoom.value;
  let tile_y =
    Math.round(
      ($event.pageY - bounding_rect.top) /
        (tile_height.value * camera_zoom.value),
    ) *
    tile_height.value *
    camera_zoom.value;
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
  left: calc(var(--selected-tile-pos-x) + var(--map-offset-x));
  top: calc(var(--selected-tile-pos-y) + var(--map-offset-y));
  width: var(--tile-width);
  height: var(--tile-height);
  border: 1px solid blue;
  position: absolute;
}
.map-editor--grid {
  background-size: var(--tile-width) var(--tile-height);
  background-position: var(--map-offset-x) var(--map-offset-y);
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
