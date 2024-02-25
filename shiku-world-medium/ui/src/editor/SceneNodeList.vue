<template>
  <div
    class="node-container"
    :class="{ 'node-container--selected': node === selected_node }"
    ref="root"
    @click="on_node_click($event, node)"
  >
    <component :is="get_game_node_settings_component(node_type)"></component>
    <div v-for="n in game_node.children">
      <SceneNodeList
        @node-selected="emit_node_selected(n)"
        :node="n"
      ></SceneNodeList>
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
import { computed, defineAsyncComponent, ref, Ref, toRefs } from "vue";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import type { GameNode } from "@/editor/blueprints/GameNode";

const props = defineProps<{ node: GameNodeKind }>();
const { node } = toRefs(props);
const emit = defineEmits<{
  (e: "nodeSelected", value: GameNodeKind): void;
}>();

const selected_node: Ref<GameNodeKind | undefined> = ref();
const root = ref(null);

function on_node_click($event: MouseEvent, node: GameNodeKind) {
  if ($event.target === root.value) {
    selected_node.value = node;
    emit("nodeSelected", node);
  }
}

function emit_node_selected(node: GameNodeKind) {
  selected_node.value = node;
  emit("nodeSelected", node);
}
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
