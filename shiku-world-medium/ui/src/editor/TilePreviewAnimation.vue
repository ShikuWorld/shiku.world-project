<template>
  <TilePreview :tileset="tileset" :tile_id="current_tile_id"></TilePreview>
</template>
<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref, toRefs } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { CharacterAnimationState } from "@/editor/blueprints/CharacterAnimationState";
import { CharacterDirection } from "@/editor/blueprints/CharacterDirection";
import TilePreview from "@/editor/editor/TilePreview.vue";

interface Props {
  tileset: Tileset;
  character_direction: CharacterDirection;
  animation_state: CharacterAnimationState;
}

const props = defineProps<Props>();
let frame_timeout: number | null = null;
const { tileset, character_direction, animation_state } = toRefs(props);

let current_frame_index = ref(0);

const current_tile_id = computed(() => {
  const frame = animation_state.value.frames[current_frame_index.value];
  return frame && frame.gid_map[character_direction.value]
    ? frame.gid_map[character_direction.value]
    : 0;
});

onMounted(() => {
  if (animation_state.value.frames.length > 0) {
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
  const frame_duration = animation_state.value.frames[current_frame_index.value]
    ? animation_state.value.frames[current_frame_index.value].duration_in_ms
    : 100;
  frame_timeout = setTimeout(() => {
    current_frame_index.value =
      (current_frame_index.value + 1) % animation_state.value.frames.length;
    animate_to_next_frame();
  }, frame_duration);
};
</script>
