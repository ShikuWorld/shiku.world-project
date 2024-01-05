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

const { set_tile_brush } = use_editor_store();
const { tile_brush } = storeToRefs(use_editor_store());
const tab = ref<string>();

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
  set_tile_brush(g_ids);
}

function reset_brush() {
  set_tile_brush([[0]]);
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
