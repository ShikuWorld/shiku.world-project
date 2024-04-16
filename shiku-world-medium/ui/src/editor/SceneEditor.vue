<template>
  <div class="entities-list">
    <v-btn
      :icon="mdiPlus"
      :id="menu_id"
      density="comfortable"
      color="primary"
      size="small"
    >
    </v-btn>
    <v-menu :activator="menu_id_hash">
      <v-list>
        <v-list-item v-for="node_type in node_type_options">
          <v-list-item-title
            v-if="
              selected_node.selection_path &&
              selected_node.selected_game_node_id
            "
            @click="
              add_node_type(
                selected_node.selection_path,
                selected_node.selected_game_node_id,
                selected_node.selected_entity_id !== undefined
                  ? selected_node.selected_entity_id
                  : null,
                node_type.value,
              )
            "
            >{{ node_type.label }}
          </v-list-item-title>
        </v-list-item>
      </v-list>
    </v-menu>
    <SceneNodeList
      :scene_resource_path="scene_key(scene)"
      :node="scene.root_node"
      :path="[]"
      :node_is_instance="is_node_instance"
      :scene_is_instance="is_scene_instance"
      @remove_node="on_remove_node"
    ></SceneNodeList>
  </div>
</template>

<style>
.entities-list {
  padding: 10px;
}
</style>

<script lang="ts" setup>
import { computed, toRefs } from "vue";
import type { Scene } from "@/editor/blueprints/Scene";
import SceneNodeList from "@/editor/editor/SceneNodeList.vue";
import { mdiPlus } from "@mdi/js";
import { storeToRefs } from "pinia";
import { use_inspector_store } from "@/editor/stores/inspector";
import {
  create_game_node,
  GameNodeTypeKeys,
  scene_key,
} from "@/editor/stores/resources";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { Entity } from "@/editor/blueprints/Entity";

const node_type_options: { value: GameNodeTypeKeys; label: string }[] = [
  { value: "Instance", label: "Instance" },
  { value: "Node2D-Node2D", label: "Node 2D" },
  { value: "Node2D-RigidBody", label: "Node 2D RigidBody" },
  { value: "Node2D-Render", label: "Node 2D Render" },
  { value: "Node2D-Collider", label: "Node 2D Collider" },
];
const props = defineProps<{
  scene: Scene;
  is_scene_instance: boolean;
  menu_id: string;
}>();
const { scene, is_scene_instance, menu_id } = toRefs(props);

const menu_id_hash = computed(() => `#${menu_id.value}`);

const { component_stores } = storeToRefs(use_inspector_store());
const selected_node = computed(() => component_stores.value.game_node);
const is_node_instance = computed(
  () => component_stores.value.game_node.is_instance === true,
);
const emit = defineEmits<{
  (
    e: "remove_node",
    scene_resource: string,
    path: number[],
    node: GameNodeKind,
    is_from_current_instance: boolean,
  ): void;
  (
    e: "add_node",
    scene_resource: string,
    path: number[],
    parent_game_node_id: string,
    parent_node_entity_id: Entity | null,
    node: GameNodeKind,
    is_from_current_instance: boolean,
  ): void;
}>();
function on_remove_node(
  scene_resource: string,
  path: number[],
  node: GameNodeKind,
  is_from_current_instance: boolean,
) {
  emit("remove_node", scene_resource, path, node, is_from_current_instance);
}

function add_node_type(
  path: number[],
  selected_game_node_id: string,
  selected_entity_id: Entity | null,
  node_type: GameNodeTypeKeys,
) {
  if (!path) {
    console.error("Tried to add node to undefined node.");
    return;
  }
  let game_node = create_game_node(node_type);
  if (!game_node) {
    console.error("Could not create game node to add to scene on server!");
    return;
  }
  emit(
    "add_node",
    scene_key(scene.value),
    path,
    selected_game_node_id,
    selected_node.value.selected_entity_id !== undefined
      ? selected_node.value.selected_entity_id
      : null,
    game_node,
    is_scene_instance.value,
  );
}
</script>
