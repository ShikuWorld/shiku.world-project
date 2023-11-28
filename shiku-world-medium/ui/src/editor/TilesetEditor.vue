<template>
  <div>
    <div class="tileset-editor__main-image" v-if="tileset.image">
      <div class="tileset-editor__main-image-backdrop"><img :src="img" /></div>
      <div class="tileset-editor__tiles">
        <div class="tileset-editor__tile-row" v-for="y of columns">
          <div
            class="tileset-editor__tile"
            v-for="x of rows"
            @click="select_tile(y, x)"
            :key="g_id(y, x)"
            :class="{
              'tileset-editor__tile--active': selected_tile === g_id(y, x),
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
import { tileset_key, use_editor_store } from "@/editor/stores/editor";
const { set_selected_tile, set_sidebar_editor } = use_editor_store();
const { resource_base_url } = use_config_store();
const props = defineProps<{ tileset: Tileset }>();
const selected_tile = ref(0);
const { tileset } = toRefs(props);
function select_tile(y: number, x: number) {
  selected_tile.value = g_id(y, x);
  set_selected_tile(tileset_key(tileset.value), g_id(y, x));
  set_sidebar_editor("tile");
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
