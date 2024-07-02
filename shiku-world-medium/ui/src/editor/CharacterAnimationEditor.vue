<template>
  <div class="animations">
    <VueFlow
      class="flow"
      :nodes="nodes"
      :edges="edges"
      @nodeClick="on_node_selected"
    >
      <template #node-custom="props">
        <div class="custom-node">
          {{ props.label }}
          <TilePreviewAnimation
            v-if="
              character_animation_tileset &&
              character_animation.states[Number(props.id)]
            "
            :tileset="character_animation_tileset"
            :character_direction="character_animation.start_direction"
            :animation_state="character_animation.states[Number(props.id)]"
          ></TilePreviewAnimation>
        </div>
      </template>
    </VueFlow>
  </div>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";
import { VueFlow, useVueFlow, NodeMouseEvent } from "@vue-flow/core";
import { CharacterAnimation } from "@/editor/blueprints/CharacterAnimation";
import { use_resources_store } from "@/editor/stores/resources";
import TilePreviewAnimation from "@/editor/editor/TilePreviewAnimation.vue";
import { storeToRefs } from "pinia";

function on_node_selected(event: NodeMouseEvent) {
  emit("select_animation_node", parseInt(event.node.id));
}

const emit = defineEmits<{
  (e: "select_animation_node", node_id: number): void;
}>();

const props = defineProps<{
  character_animation: CharacterAnimation;
}>();

const { character_animation } = toRefs(props);
const { get_or_load_tileset } = use_resources_store();

const { tileset_map } = storeToRefs(use_resources_store());

const character_animation_tileset = computed(() =>
  get_or_load_tileset(
    tileset_map.value,
    character_animation.value.tileset_resource,
  ),
);

const nodes = computed(() => {
  return Object.entries(character_animation.value.states).map(
    ([key, state]) => ({
      id: key,
      label: state.name,
      type: "custom",
      position: { x: Math.random() * 400, y: Math.random() * 400 },
    }),
  );
});

const edges = computed(() => {
  return [];
});

/*function transition_key(
  transitionId: number | string,
  from: number | string,
  to: number | string,
) {
  return `${transitionId}-${from}-${to}`;
}*/

const { findNode, onInit } = useVueFlow();

onInit(() => {
  const node = findNode("0");

  if (node) {
    node.position = { x: 100, y: 100 };
  }
});
</script>

<style>
@import "@vue-flow/core/dist/style.css";
@import "@vue-flow/core/dist/theme-default.css";

.animations {
  pointer-events: all;
  width: 1400px;
  height: 800px;
}
.custom-node {
  background-color: #1a192b;
  padding: 16px;
  border-radius: 4px;
}
</style>
