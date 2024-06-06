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
        @update:model-value="select_tile_layer"
        :items="tile_layers"
      ></v-select>
      <v-number-input
        label="Parralax X"
        control-variant="stacked"
        :step="0.01"
        :model-value="parralax_x"
        @update:model-value="
          (new_value) =>
            set_parralax(selected_tile_layer, Number(new_value), parralax_y)
        "
      ></v-number-input>
      <v-number-input
        label="Parralax Y"
        control-variant="stacked"
        :step="0.01"
        :model-value="parralax_y"
        @update:model-value="
          (new_value) =>
            set_parralax(selected_tile_layer, parralax_x, Number(new_value))
        "
      ></v-number-input>
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
import { VNumberInput } from "vuetify/labs/VNumberInput";
import TilesetEditor from "@/editor/editor/TilesetEditor.vue";
import { mdiEraserVariant } from "@mdi/js";
import { storeToRefs } from "pinia";
import { match } from "ts-pattern";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { GameMap } from "@/editor/blueprints/GameMap";
import { use_resources_store } from "@/editor/stores/resources";

type BrushSize = "1x1" | "2x2" | "3x3" | "5x5";
const { set_tile_brush, set_selected_tile_layer } = use_editor_store();
const { tile_brush, selected_tile_layer, current_main_instance } =
  storeToRefs(use_editor_store());
const { update_map_server } = use_resources_store();
const { game_map_map } = storeToRefs(use_resources_store());

const current_main_map = computed<GameMap | undefined>(() => {
  if (current_main_instance.value?.world_id && game_map_map.value) {
    return Object.values(game_map_map.value).find(
      (m) => m.world_id === current_main_instance.value.world_id,
    );
  }
  return undefined;
});

function select_tile_layer(layer: LayerKind) {
  set_selected_tile_layer(layer);
  update_main_instance_grid_p_scaling(layer);
}

function update_main_instance_grid_p_scaling(layer: LayerKind) {
  if (
    current_main_instance.value &&
    current_main_instance.value.instance_id &&
    current_main_instance.value.world_id
  ) {
    window.medium.sync_grid_with_layer_p_scaling(
      current_main_instance.value.instance_id,
      current_main_instance.value.world_id,
      layer,
    );
  }
}

const tab = ref<string>();
function set_parralax(
  layer: LayerKind,
  parralax_x: number,
  parralax_y: number,
) {
  if (current_main_map.value) {
    update_map_server({
      name: current_main_map.value.name,
      resource_path: current_main_map.value.resource_path,
      chunk: null,
      scene: null,
      layer_parallax: [layer, [parralax_x, parralax_y]],
    });
    update_main_instance_grid_p_scaling(selected_tile_layer.value);
  }
}

const parralax_x = computed(() => {
  if (current_main_map.value) {
    return current_main_map.value.layer_parallax[selected_tile_layer.value][0];
  }
  return 1.0;
});
const parralax_y = computed(() => {
  if (current_main_map.value) {
    return current_main_map.value.layer_parallax[selected_tile_layer.value][1];
  }
  return 1.0;
});
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
