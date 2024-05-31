<template>
  <div class="node-container">
    <v-btn v-if="instance_resource_path" @click="save_back_to_instance_scene"
      >Update Blueprint</v-btn
    >
    <component
      :is="node_component"
      v-bind="{ game_node, is_instance }"
      @entityUpdate="entity_update"
      :key="game_node.id"
    ></component>
    <v-label v-if="game_node.script && is_instance" class="form-label"
      >Scope variables</v-label
    >
    <div v-if="game_node.script && is_instance">
      <div
        v-for="[scope_key, scope_type, scope_value] in scope_cache"
        :key="scope_key"
      >
        <v-label class="form-label">{{ scope_key }}</v-label>
        <v-text-field
          v-if="scope_type === 'String'"
          :model-value="scope_value"
          @update:model-value="
            (new_value) => scope_update(scope_key, new_value, scope_type)
          "
        ></v-text-field>
        <v-text-field
          v-if="scope_type === 'Number'"
          type="number"
          :model-value="scope_value"
          @update:model-value="
            (new_value) => scope_update(scope_key, new_value, scope_type)
          "
        ></v-text-field>
        <v-text-field
          v-if="scope_type === 'Integer'"
          type="number"
          :model-value="scope_value"
          @update:model-value="
            (new_value) => scope_update(scope_key, new_value, scope_type)
          "
        ></v-text-field>
      </div>
    </div>
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
import { use_game_instances_store } from "@/editor/stores/game-instances";
import { ScopeCacheValue } from "@/editor/blueprints/ScopeCacheValue";

const props = defineProps<{
  scene_resource_path: string;
  node: GameNodeKind;
  path: number[];
  is_instance: boolean;
}>();
const { node, path, scene_resource_path, is_instance } = toRefs(props);

const node_type = computed(() => get_game_node_type(node.value));
const game_node = computed(() => get_generic_game_node(node.value));
const instance_resource_path = computed(() => {
  return game_node.value && game_node.value.instance_resource_path
    ? game_node.value.instance_resource_path
    : null;
});
const {
  update_instance_node,
  update_data_in_scene_node_on_server,
  update_scene_root_with_node,
} = use_resources_store();

const { game_instance_data_map } = storeToRefs(use_game_instances_store());
const { selected_module_id, current_main_instance } =
  storeToRefs(use_editor_store());

const save_back_to_instance_scene = () => {
  if (instance_resource_path.value) {
    update_scene_root_with_node(instance_resource_path.value, node.value);
  }
};

const scope_cache = computed<Array<[string, string, string | number]> | null>(
  () => {
    const instance_id = current_main_instance.value.instance_id;
    const world_id = current_main_instance.value.world_id;
    if (current_main_instance.value && instance_id && world_id) {
      if (
        game_instance_data_map.value[instance_id] &&
        game_instance_data_map.value[instance_id][world_id]
      ) {
        const game_instance_data =
          game_instance_data_map.value[instance_id][world_id];
        console.log(game_node.value.entity_id, game_instance_data.scope_cache);
        if (
          typeof game_node.value.entity_id === "number" &&
          game_instance_data.scope_cache[game_node.value.entity_id]
        ) {
          return Object.entries(
            game_instance_data.scope_cache[game_node.value.entity_id],
          ).map(([key, value]) => [
            key,
            Object.keys(value)[0],
            Object.values(value)[0],
          ]);
        }
      }
    }
    return null;
  },
);

function scope_update(key: string, value: string | number, type: string) {
  if (type === "Number") {
    value = Number(value);
    if (Number.isNaN(value)) {
      return;
    }
  }
  if (type === "Integer") {
    value = Math.round(Number(value));
    if (Number.isNaN(value)) {
      return;
    }
  }
  entity_update({
    UpdateScriptScope: [key, { [type]: value } as ScopeCacheValue],
  });
}

function entity_update(entity_update: EntityUpdateKind) {
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
