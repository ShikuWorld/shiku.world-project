<template>
  <div v-if="loading">Loading</div>
  <div v-else class="tile-selector">
    <div class="tile-selector__eraser">
      <v-btn
        :icon="mdiEraserVariant"
        @click="reset_brush"
        :flat="true"
        variant="flat"
        :active="eraser_active"
      ></v-btn>
      <v-select
        label="Size"
        :model-value="selected_brush_size"
        @update:model-value="set_selected_brush_size"
        :items="brush_sizes"
      ></v-select>
      <v-select
        label="Layer"
        :model-value="selected_tile_layer"
        @update:model-value="set_selected_tile_layer"
        :items="tile_layers"
      ></v-select>
    </div>
    <v-tabs v-model="tab" bg-color="primary">
      <v-tab
        v-for="tileset in tilesets"
        :key="tileset_key(tileset)"
        :value="tileset_key(tileset)"
        >{{ tileset.name }}</v-tab
      >
    </v-tabs>
    <v-window v-model="tab">
      <v-window-item
        v-for="tileset in tilesets"
        :key="tileset_key(tileset)"
        :value="tileset_key(tileset)"
      >
        <div class="tile-selector__brush">
          <TilesetEditor
            :tileset="tileset"
            :enable_multi_tile_selection="true"
            @tile_selection="on_tile_selection"
          ></TilesetEditor>
        </div>
      </v-window-item>
    </v-window>
  </div>
</template>
<script lang="ts" setup>
import { computed, onMounted, ref, toRefs, watch } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { Point, tileset_key, use_editor_store } from "@/editor/stores/editor";
import TilesetEditor from "@/editor/editor/TilesetEditor.vue";
import { mdiEraserVariant } from "@mdi/js";
import { storeToRefs } from "pinia";
import { match } from "ts-pattern";
import { LayerKind } from "@/editor/blueprints/LayerKind";

type BrushSize = "1x1" | "2x2" | "3x3" | "5x5";
const { set_tile_brush, set_selected_tile_layer } = use_editor_store();
const { tile_brush, selected_tile_layer } = storeToRefs(use_editor_store());
const tab = ref<string>();
const selected_brush_size = ref<BrushSize>("1x1");
const brush_sizes: BrushSize[] = ["1x1", "2x2", "3x3", "5x5"];
const tile_layers: LayerKind[] = [
  "BG10",
  "BG09",
  "BG08",
  "BG07",
  "BG06",
  "BG05",
  "BG04",
  "BG03",
  "BG02",
  "BG01",
  "BG00",
  "ObjectsBelow",
  "Terrain",
  "ObjectsFront",
  "FG00",
  "FG01",
  "FG02",
  "FG03",
  "FG04",
  "FG05",
  "FG06",
  "FG07",
  "FG08",
  "FG09",
  "FG10",
];
function set_selected_brush_size(size: BrushSize) {
  selected_brush_size.value = size;
  if (gids_inside_brush.value.length === 1) {
    const single_gid = gids_inside_brush.value[0];
    set_tile_brush(
      create_brush_with_gid(single_gid, brush_size_as_number.value),
    );
  }
}

const gids_inside_brush = computed(() => {
  const gids = new Set<number>();
  for (const row of tile_brush.value) {
    for (const gid of row) {
      gids.add(gid);
    }
  }
  return Array.from(gids);
});

const brush_size_as_number = computed(() =>
  match(selected_brush_size.value)
    .with("1x1", () => 1)
    .with("2x2", () => 2)
    .with("3x3", () => 3)
    .with("5x5", () => 5)
    .exhaustive(),
);

function create_brush_with_gid(gid: number, size: number): number[][] {
  const brush: number[][] = [];
  for (let i = 0; i < size; i++) {
    brush.push([]);
    for (let j = 0; j < size; j++) {
      brush[i].push(gid);
    }
  }
  return brush;
}

const props = defineProps<{ tilesets: Tileset[] }>();
const { tilesets } = toRefs(props);

const loading = computed(() => tilesets.value.some((t) => !t));
const eraser_active = computed(
  () =>
    tile_brush.value.length == 1 &&
    tile_brush.value[0].length == 1 &&
    tile_brush.value[0][0] === 0,
);

watch(tilesets, () => {
  if (!loading.value) {
    tab.value = tileset_key(props.tilesets[0]);
  }
});

onMounted(() => {
  if (!loading.value) {
    tab.value = tileset_key(props.tilesets[0]);
  }
});

function on_tile_selection(
  _start: Point,
  _end: Point,
  g_ids: number[][],
  _tileset_key: string,
) {
  selected_brush_size.value = "1x1";
  set_tile_brush(g_ids);
}

function reset_brush() {
  set_tile_brush(create_brush_with_gid(0, brush_size_as_number.value));
}

watch(tilesets, () => {
  if (!tab.value && tilesets.value.length > 0) {
    tab.value = tileset_key(props.tilesets[0]);
  }
});
</script>
<style>
.tile-selector__brush {
  width: 100%;
  overflow-x: scroll;
}

.tile-selector__eraser {
  padding: 0 16px;
}
</style>
