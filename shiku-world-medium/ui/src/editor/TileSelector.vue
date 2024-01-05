<template>
  <div v-if="loading">Loading</div>
  <div v-else class="tile-selector">
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
        <TilesetEditor
          :tileset="tileset"
          :enable_multi_tile_selection="true"
          @tile_selection="on_tile_selection"
        ></TilesetEditor>
      </v-window-item>
    </v-window>
  </div>
</template>
<script lang="ts" setup>
import { computed, onMounted, ref, toRefs, watch } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { Point, tileset_key, use_editor_store } from "@/editor/stores/editor";
import TilesetEditor from "@/editor/editor/TilesetEditor.vue";

const { set_tile_brush } = use_editor_store();

const tab = ref<string>();

const props = defineProps<{ tilesets: Tileset[] }>();
const { tilesets } = toRefs(props);

const loading = computed(() => tilesets.value.some((t) => !t));

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

watch(tilesets, () => {
  if (!tab.value && tilesets.value.length > 0) {
    tab.value = tileset_key(props.tilesets[0]);
  }
});
</script>
<style></style>
