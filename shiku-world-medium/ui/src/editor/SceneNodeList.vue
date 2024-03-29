<template>
  <div class="node-container">
    <div
      class="node-component"
      :class="{ 'node-container--selected': game_node.id === selected_node_id }"
      @click="on_node_click($event, game_node)"
      @contextmenu="on_context_menu($event)"
      ref="comp"
    >
      {{ game_node.name }}
    </div>
    <div v-for="(n, index) in game_node.children">
      <scene-node-list
        :scene_resource_path="scene_resource_path"
        :node="n"
        :path="[...path, index]"
      ></scene-node-list>
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
import {
  get_generic_game_node,
  use_resources_store,
} from "@/editor/stores/resources";
import ContextMenu from "@imengyu/vue3-context-menu";

const props = defineProps<{
  node: GameNodeKind;
  scene_resource_path: string;
  path: number[];
}>();
const { node, path, scene_resource_path } = toRefs(props);
const game_node = computed(() => get_generic_game_node(node.value));
const comp = ref<HTMLElement>();
const { component_stores } = storeToRefs(use_inspector_store());
const selected_node_id = computed(() => {
  return component_stores.value.game_node.selected_game_node_id;
});

const { select_game_node } = use_inspector_store();
const { remove_child_from_scene_on_server } = use_resources_store();

const on_context_menu = (e: MouseEvent) => {
  prevent_browser_default(e);

  ContextMenu.showContextMenu({
    theme: "dark",
    x: e.x,
    y: e.y,
    items: [
      {
        label: "Delete",
        disabled: path.value.length === 0,
        onClick: () => {
          remove_child_from_scene_on_server(
            scene_resource_path.value,
            path.value,
            node.value,
          );
        },
      },
    ],
  });
};

function prevent_browser_default(e: MouseEvent) {
  e.preventDefault();
}

function on_node_click($event: MouseEvent, game_node: GameNode<unknown>) {
  if (
    comp.value &&
    (comp.value === ($event.target as HTMLElement) ||
      $event.target === comp.value.children[0])
  ) {
    select_game_node(scene_resource_path.value, game_node.id, path.value);
  }
}
</script>
