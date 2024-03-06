<template>
  <div class="node-container">
    <div
      class="node-component"
      :class="{ 'node-container--selected': game_node.id === selected_node_id }"
      @click="on_node_click($event, game_node)"
      ref="comp"
    >
      {{ game_node.name }}
    </div>
    <div v-for="(n, index) in game_node.children">
      <SceneNodeList :node="n" :path="[...path, index]"></SceneNodeList>
    </div>
  </div>
</template>

<style>
.node-container {
  cursor: pointer;

  padding: 10px;
}
.node-container--selected {
  background-color: red;
}
</style>

<script lang="ts" setup>
import { computed, ref, toRefs } from "vue";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import type { GameNode } from "@/editor/blueprints/GameNode";
import { use_inspector_store } from "@/editor/stores/inspector";
import { storeToRefs } from "pinia";

const props = defineProps<{ node: GameNodeKind; path: number[] }>();
const { node, path } = toRefs(props);

const comp = ref<HTMLElement>();
const { component_stores } = storeToRefs(use_inspector_store());
const selected_node_id = computed(
  () => component_stores.value.game_node.selected_game_node_id,
);

const { select_game_node } = use_inspector_store();
function on_node_click($event: MouseEvent, game_node: GameNode<unknown>) {
  if (
    comp.value &&
    (comp.value === $event.target || $event.target === comp.value.children[0])
  ) {
    select_game_node(game_node.id, path.value);
  }
}

const game_node = computed(
  () => Object.values(node.value)[0] as GameNode<unknown>,
);
</script>
