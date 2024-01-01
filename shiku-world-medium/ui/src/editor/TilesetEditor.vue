<template>
  <div>
    <div class="tileset-editor__main-image" v-if="tileset.image">
      <div class="tileset-editor__main-image-backdrop"><img :src="img" /></div>
      <div class="tileset-editor__tiles">
        <div class="tileset-editor__tile-row" v-for="y of columns">
          <div
            class="tileset-editor__tile"
            v-for="x of rows"
            @mousedown="start_selection(y, x)"
            @mouseenter="move_selection_end(y, x)"
            @mouseup="end_selection(y, x)"
            :key="g_id(y, x)"
            :class="{
              'tileset-editor__tile--active': selected_tiles.has(g_id(y, x)),
            }"
            :style="{
              width: `${tileset.tile_width}px`,
              height: `${tileset.tile_height}px`,
            }"
          ></div>
        </div>
      </div>
    </div>
  </div>
</template>
<script lang="ts" setup>
import { computed, ref, toRefs } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { use_config_store } from "@/editor/stores/config";
import { Point, tileset_key } from "@/editor/stores/editor";

const { resource_base_url } = use_config_store();
const props = defineProps<{
  tileset: Tileset;
  enable_multi_tile_selection?: boolean;
}>();
const sel_start = ref({ y: 0, x: 0 });
const sel_end = ref({ y: 0, x: 0 });
let sel_running = false;
const selected_tiles = computed(() => {
  let set = new Set();
  if (sel_start.value) {
    for (let y_i = sel_start.value.y; y_i <= sel_end.value.y; y_i++) {
      for (let x_i = sel_start.value.x; x_i <= sel_end.value.x; x_i++) {
        set.add(g_id(y_i, x_i));
      }
    }
  }
  return set;
});
const { tileset, enable_multi_tile_selection } = toRefs(props);
const emit = defineEmits<{
  (
    e: "tile_selection",
    start: Point,
    end: Point,
    g_ids: number[][],
    tileset_key: string,
  ): void;
  (e: "tile_selected", g_id: number, tileset_key: string): void;
}>();
function start_selection(y: number, x: number) {
  sel_start.value = { y, x };
  sel_end.value = { y, x };
  sel_running = true;
}

function move_selection_end(y: number, x: number) {
  if (sel_running && enable_multi_tile_selection.value) {
    sel_end.value = { y, x };
  }
}

function end_selection(y: number, x: number) {
  sel_end.value = { y, x };
  if (!enable_multi_tile_selection.value) {
    sel_start.value = { y, x };
    emit("tile_selected", g_id(y, x), tileset_key(tileset.value));
  } else {
    const gid_selection = [];
    for (let y_i = sel_start.value.y; y_i <= sel_end.value.y; y_i++) {
      const gid_selection_columns = [];
      for (let x_i = sel_start.value.x; x_i <= sel_end.value.x; x_i++) {
        gid_selection_columns.push(g_id(y_i, x_i));
      }
      gid_selection.push(gid_selection_columns);
    }
    emit(
      "tile_selection",
      sel_start.value,
      sel_end.value,
      gid_selection,
      tileset_key(tileset.value),
    );
  }
  sel_running = false;
}

const img = computed(
  () =>
    `${resource_base_url}${
      tileset.value.image?.path ? tileset.value.image.path : ""
    }`,
);

const rows = computed(() => {
  if (tileset.value.image) {
    return tileset.value.image.width / tileset.value.tile_width;
  }
  return 0;
});
function g_id(y: number, x: number) {
  return (y - 1) * rows.value + x;
}
const columns = computed(() => {
  if (tileset.value.image) {
    return tileset.value.image.height / tileset.value.tile_height;
  }
  return 0;
});
</script>
<style>
.tileset-editor__main-image {
  position: relative;
}
.tileset-editor__tiles {
  position: absolute;
  top: 0;
  left: 0;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  font-size: 8px;
}
.tileset-editor__tile-row {
  display: flex;
}
.tileset-editor__tile {
  display: inline-block;
  border: 1px dashed rgba(9, 223, 181, 0.23);
  box-sizing: border-box;
  mix-blend-mode: difference;
}
.tileset-editor__tile--active {
  background-color: rgba(200, 200, 200, 0.8);
  border: 1px solid rgb(185, 9, 244);
  mix-blend-mode: hard-light;
}

.tileset-editor__tile:hover {
  background-color: rgba(0, 0, 0, 0.2);
  border: 1px dashed rgba(255, 255, 255, 0.5);
  box-sizing: border-box;
}
.tileset-editor__tile--active:hover {
  background-color: rgba(107, 151, 230, 0.8);
  border: 1px dashed rgba(255, 255, 255, 1);
}
</style>
