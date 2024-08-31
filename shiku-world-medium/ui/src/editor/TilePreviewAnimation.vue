<template>
  <TilePreview :tileset="tileset" :tile_id="current_tile_id"></TilePreview>
</template>
<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref, toRefs, watch } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import TilePreview from "@/editor/editor/TilePreview.vue";

interface Props {
  tileset: Tileset;
  tile_id: number;
}

const props = defineProps<Props>();
let frame_timeout: number | null = null;
const { tileset, tile_id } = toRefs(props);

let current_frame_index = ref(0);

const tile_animation = computed(() => {
  return tileset.value.tiles[tile_id.value]?.animation ?? [];
});

const current_tile_id = computed(() => {
  return tile_animation.value[current_frame_index.value]?.id ?? 0;
});

watch([tileset, tile_id], () => {
  if (frame_timeout === null) {
    animate_to_next_frame();
  }
});

onMounted(() => {
  if (tile_animation.value.length > 0) {
    animate_to_next_frame();
  }
});

onUnmounted(() => {
  if (frame_timeout !== null) {
    clearTimeout(frame_timeout);
    frame_timeout = null;
  }
});

const animate_to_next_frame = () => {
  if (frame_timeout !== null) {
    clearTimeout(frame_timeout);
    frame_timeout = null;
  }
  const frame_duration = tile_animation.value[current_frame_index.value]
    ? tile_animation.value[current_frame_index.value].duration
    : 100;
  frame_timeout = window.setTimeout(() => {
    current_frame_index.value =
      (current_frame_index.value + 1) % tile_animation.value.length;
    animate_to_next_frame();
  }, frame_duration);
};
</script>
