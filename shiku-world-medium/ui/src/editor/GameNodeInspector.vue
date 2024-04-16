<template>
  <div class="node-container">
    <component
      :is="node_component"
      v-bind="{ game_node }"
      @entityUpdate="entity_update"
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
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { storeToRefs } from "pinia";
import { use_editor_store } from "@/editor/stores/editor";

const props = defineProps<{
  scene_resource_path: string;
  node: GameNodeKind;
  path: number[];
  is_instance: boolean;
}>();
const { node, path, scene_resource_path, is_instance } = toRefs(props);

const node_type = computed(() => get_game_node_type(node.value));
const game_node = computed(() => get_generic_game_node(node.value));
const { update_instance_node, update_data_in_scene_node_on_server } =
  use_resources_store();

const { selected_module_id, current_main_instance } =
  storeToRefs(use_editor_store());

function entity_update(entity_update: EntityUpdateKind) {
  console.log(game_node.value, game_node.value.entity_id);
  if (!is_instance.value && path.value && scene_resource_path.value) {
    update_data_in_scene_node_on_server(
      scene_resource_path.value,
      path.value,
      game_node.value.id,
      entity_update,
    );
  } else if (
    selected_module_id.value &&
    current_main_instance.value &&
    current_main_instance.value.instance_id !== undefined &&
    current_main_instance.value.world_id !== undefined &&
    game_node.value &&
    game_node.value.entity_id !== null
  ) {
    console.log("???");
    update_instance_node(
      selected_module_id.value,
      current_main_instance.value.instance_id,
      current_main_instance.value.world_id,
      { id: game_node.value.entity_id, kind: entity_update },
    );
  }
}

const node_component = computed(() => {
  return defineAsyncComponent(
    () => import(/* @vite-ignore */ `./game_nodes/${node_type.value}.vue`),
  );
});
</script>
