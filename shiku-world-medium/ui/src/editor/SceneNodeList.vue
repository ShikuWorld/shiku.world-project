<template>
  <div class="node-container">
    <div
      class="node-component"
      :class="{ 'node-container--selected': node === selected_node }"
      @click="on_node_click($event, node)"
      ref="comp"
    >
      {{ game_node.name }}
    </div>
    <div v-for="n in game_node.children">
      <SceneNodeList :node="n"></SceneNodeList>
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

const props = defineProps<{ node: GameNodeKind }>();
const { node } = toRefs(props);

const comp = ref<HTMLElement>();
const { component_stores } = storeToRefs(use_inspector_store());
const selected_node = computed(
  () => component_stores.value.game_node.selected_game_node,
);

const { select_game_node } = use_inspector_store();
function on_node_click($event: MouseEvent, node: GameNodeKind) {
  console.log(comp.value, $event.target);
  if (
    comp.value &&
    (comp.value === $event.target || $event.target === comp.value.children[0])
  ) {
    select_game_node(node);
  }
}

const game_node = computed(
  () => Object.values(node.value)[0] as GameNode<unknown>,
);
</script>
