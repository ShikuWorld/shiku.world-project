<template>
  <div class="node-container">
    <component
      :is="node_component"
      v-bind="{ game_node }"
      @dataUpdate="dataUpdated"
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
  use_resources_store,
} from "@/editor/stores/resources";
import { GameNode } from "@/editor/blueprints/GameNode";

const props = defineProps<{
  scene_resource_path: string;
  node: GameNodeKind;
  path: number[];
}>();
const { node, path, scene_resource_path } = toRefs(props);

const node_type = computed(() => get_game_node_type(node.value));
const game_node = computed(() => get_generic_game_node(node.value));
const { update_data_in_scene_node_on_server } = use_resources_store();

function dataUpdated(data: unknown) {
  const game_node_update: GameNode<unknown> = {
    id: game_node.value.id,
    name: game_node.value.name,
    script: game_node.value.script,
    children: [],
    data,
  };
  update_data_in_scene_node_on_server(scene_resource_path.value, path.value, {
    [node_type.value]: game_node_update,
  } as unknown as GameNodeKind);
}

const node_component = computed(() => {
  return defineAsyncComponent(
    () => import(/* @vite-ignore */ `./game_nodes/${node_type.value}.vue`),
  );
});
</script>
