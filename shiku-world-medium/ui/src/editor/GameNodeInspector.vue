<template>
  <div class="node-container">
    <component
      :is="get_game_node_settings_component(node_type)"
      :key="game_node.id"
    ></component>
  </div>
</template>

<style>
.node-container {
  cursor: pointer;

  padding: 10px;
}
</style>

<script lang="ts" setup>
import { computed, defineAsyncComponent, toRefs } from "vue";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import type { GameNode } from "@/editor/blueprints/GameNode";

const props = defineProps<{ node: GameNodeKind }>();
const { node } = toRefs(props);

const node_type = computed(() => Object.keys(node.value)[0]);
const game_node = computed(
  () => Object.values(node.value)[0] as GameNode<unknown>,
);

function get_game_node_settings_component(component_name: string) {
  return defineAsyncComponent(
    () => import(/* @vite-ignore */ `./game_nodes/${component_name}.vue`),
  );
}
</script>
