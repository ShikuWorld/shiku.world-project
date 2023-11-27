<template>
  <div>
    <div class="tileset-editor__main-image" v-if="tileset.image">
      <div class="tileset-editor__main-image-backdrop"><img :src="img" /></div>
      <div class="tileset-editor__tiles">
        <div class="tileset-editor__tile-row" v-for="x of columns">
          <div
            class="tileset-editor__tile"
            v-for="y of rows"
            @click="select_tile(x, y)"
            :key="`${x},${y}`"
            :class="{
              'tileset-editor__tile--active':
                selected_tile[0] === x && selected_tile[1] === y,
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
const { resource_base_url } = use_config_store();
const props = defineProps<{ tileset: Tileset }>();
const selected_tile = ref([0, 0]);
const { tileset } = toRefs(props);
function select_tile(x: number, y: number) {
  selected_tile.value = [x, y];
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
}
.tileset-editor__tile-row {
  display: flex;
}
.tileset-editor__tile {
  display: inline-block;
  border: 1px dashed rgba(0, 0, 0, 0.2);
  box-sizing: border-box;
}
.tileset-editor__tile--active {
  background-color: rgba(100, 149, 237, 0.5);
  border: 1px dashed rgba(255, 255, 255, 0.7);
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
