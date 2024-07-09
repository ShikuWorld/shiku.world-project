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
    <div class="tileset-editor__separate_images" v-if="tileset.image === null">
      <div v-for="(tile, key, index) in tileset.tiles" :key="key">
        <div
          class="tileset-editor__tile tileset-editor__independent_tile"
          v-if="tile.image"
          @click="select_tile(tile.id)"
        >
          <v-text-field
            label="Image source"
            class="tileset-editor__source"
            :model-value="tile.image.path"
            @update:model-value="(value) => change_tile_source(value, tile.id)"
            density="compact"
            hide-details="auto"
          ></v-text-field>
          <v-img
            :src="`${resource_base_url}${tile.image.path}`"
            class="tileset-editor__preview"
            ref="previews"
            @load="image_loaded(index, tile.id)"
          ></v-img>
          <div class="tileset-editor__independent_tile_info">
            <span>{{ tile.image.width }} x {{ tile.image.height }}</span>
            <v-btn
              density="compact"
              @click="remove_tile(tile.id)"
              :icon="mdiTrashCan"
            ></v-btn>
          </div>
        </div>
      </div>
      <v-btn @click="add_tile" :icon="mdiPlus"></v-btn>
    </div>
  </div>
</template>
<script lang="ts" setup>
import { computed, ref, toRefs } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { use_config_store } from "@/editor/stores/config";
import { Point, tileset_key } from "@/editor/stores/editor";
import { use_resources_store } from "@/editor/stores/resources";
import { Tile } from "@/editor/blueprints/Tile";
import { VImg } from "vuetify/components";
import { mdiPlus, mdiTrashCan } from "@mdi/js";

const { resource_base_url } = use_config_store();
const { update_tileset_server } = use_resources_store();
const props = defineProps<{
  tileset: Tileset;
  start_gid: number;
  enable_multi_tile_selection?: boolean;
}>();
const previews = ref<VImg[]>();
function image_loaded(image_index: number, gid: number) {
  const tile = tileset.value.tiles[gid];
  const previewImage = previews.value?.[image_index]?.image;
  console.log(tile, previews.value);
  if (previewImage && tile.image) {
    update_tileset_server(tileset_key(tileset.value), {
      ChangeTileImage: [
        gid,
        {
          ...tile.image,
          ...{
            width: previewImage.naturalWidth,
            height: previewImage.naturalHeight,
          },
        },
      ],
    });
  }
}
const change_tile_source_bounce_timeout = ref<number | null>(null);
const change_tile_source = (path: string, gid: number) => {
  if (change_tile_source_bounce_timeout.value !== null) {
    clearTimeout(change_tile_source_bounce_timeout.value);
  }
  change_tile_source_bounce_timeout.value = setTimeout(() => {
    const tile = tileset.value.tiles[gid];
    if (tile && tile.image) {
      update_tileset_server(tileset_key(tileset.value), {
        ChangeTileImage: [gid, { ...tile.image, path }],
      });
    }
  }, 500);
};
const next_gid = computed(() => {
  const gids = Object.keys(tileset.value.tiles).map((x) => parseInt(x));

  let i = 0;
  while (gids.includes(i)) {
    i++;
  }
  return i;
});

const add_tile = () => {
  if (tileset.value.image === null) {
    let next_id = next_gid.value;
    let tile: Tile = {
      id: next_id,
      image: {
        height: 0,
        path: "",
        width: 0,
      },
      animation: null,
      collision_shape: null,
    };
    update_tileset_server(tileset_key(tileset.value), {
      AddTile: [next_id, tile],
    });
  }
};

const remove_tile = (gid: number) => {
  if (tileset.value.image === null) {
    update_tileset_server(tileset_key(tileset.value), {
      RemoveTile: gid,
    });
  }
};

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
const { tileset, start_gid, enable_multi_tile_selection } = toRefs(props);
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

function select_tile(gid: number) {
  emit("tile_selected", gid, tileset_key(tileset.value));
}

function end_selection(y: number, x: number) {
  sel_end.value = { y, x };
  if (!enable_multi_tile_selection.value) {
    sel_start.value = { y, x };
    select_tile(g_id(y, x));
  } else {
    const gid_selection = [];
    for (let y_i = sel_start.value.y; y_i <= sel_end.value.y; y_i++) {
      const gid_selection_columns = [];
      for (let x_i = sel_start.value.x; x_i <= sel_end.value.x; x_i++) {
        gid_selection_columns.push(g_id(y_i, x_i) + start_gid.value);
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
.tileset-editor__source {
  margin-bottom: 16px;
}
.tileset-editor__independent_tile_info {
  display: flex;
  justify-content: space-around;
  align-content: center;
  margin-top: 16px;
}
.tileset-editor__separate_images {
  display: flex;
  flex-wrap: wrap;
}
.tileset-editor__independent_tile {
  width: 200px;
  justify-content: center;
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
.tileset-editor__preview {
  width: 100%;
  height: 120px;
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
