<template>
  <div class="node-container">
    <component
      :is="get_game_node_settings_component(node_type)"
      v-bind="{ game_node }"
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
import {
  get_game_node_type,
  get_generic_game_node,
} from "@/editor/stores/resources";

const props = defineProps<{ node: GameNodeKind }>();
const { node } = toRefs(props);

const node_type = computed(() => get_game_node_type(node.value));
const game_node = computed(() => get_generic_game_node(node.value));

function get_game_node_settings_component(component_name: string) {
  return defineAsyncComponent(
    () => import(/* @vite-ignore */ `./game_nodes/${component_name}.vue`),
  );
}
</script>
