<template>
  <div class="entities-list">
    <v-btn
      v-if="!is_scene_instance"
      :icon="mdiPlus"
      id="menu-activator"
      density="comfortable"
      color="primary"
      size="small"
    >
    </v-btn>
    <v-menu activator="#menu-activator">
      <v-list>
        <v-list-item v-for="node_type in node_type_options">
          <v-list-item-title
            v-if="selected_node.selection_path"
            @click="
              add_node_type(selected_node.selection_path, node_type.value)
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
  use_resources_store,
} from "@/editor/stores/resources";

const node_type_options: { value: GameNodeTypeKeys; label: string }[] = [
  { value: "Instance", label: "Instance" },
  { value: "Node2D-Node2D", label: "Node 2D" },
  { value: "Node2D-RigidBody", label: "Node 2D RigidBody" },
  { value: "Node2D-Render", label: "Node 2D Render" },
  { value: "Node2D-Collider", label: "Node 2D Collider" },
];
const props = defineProps<{ scene: Scene; is_scene_instance: boolean }>();
const { scene, is_scene_instance } = toRefs(props);

const { component_stores } = storeToRefs(use_inspector_store());
const { add_child_to_scene_on_server } = use_resources_store();
const selected_node = computed(() => component_stores.value.game_node);
const is_node_instance = computed(
  () => component_stores.value.game_node.is_instance === true,
);
function add_node_type(path: number[], node_type: GameNodeTypeKeys) {
  if (!path) {
    console.error("Tried to add node to undefined node.");
    return;
  }
  let game_node = create_game_node(node_type);
  if (!game_node) {
    console.error("Could not create game node to add to scene on server!");
    return;
  }
  add_child_to_scene_on_server(scene_key(scene.value), path, game_node);
}
</script>
