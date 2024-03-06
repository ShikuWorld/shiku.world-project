<template>
  <div class="entities-list">
    <v-btn
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
            @click="add_node_type(selected_node.path, node_type.value)"
            >{{ node_type.label }}</v-list-item-title
          >
        </v-list-item>
      </v-list>
    </v-menu>
    <scene-node-list :node="scene.root_node" :path="[]"></scene-node-list>
  </div>
</template>

<style>
.entities-list {
  padding: 10px;
}
</style>

<script lang="ts" setup>
import { computed, toRaw, toRefs } from "vue";
import type { Scene } from "@/editor/blueprints/Scene";
import SceneNodeList from "@/editor/editor/SceneNodeList.vue";
import { mdiPlus } from "@mdi/js";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { KeysOfUnion } from "@/editor/utils";
import { storeToRefs } from "pinia";
import { use_inspector_store } from "@/editor/stores/inspector";
import { create_game_node } from "@/editor/stores/editor";
import { use_resources_store } from "@/editor/stores/resources";

const node_type_options: { value: KeysOfUnion<GameNodeKind>; label: string }[] =
  [
    { value: "Instance", label: "Instance" },
    { value: "Render", label: "Render" },
    { value: "Collider", label: "Collider" },
    { value: "RigidBody", label: "RigidBody" },
  ];
const props = defineProps<{ scene: Scene }>();
const { scene } = toRefs(props);

const { component_stores } = storeToRefs(use_inspector_store());
const { update_scene_server } = use_resources_store();
const selected_node = computed(() => component_stores.value.game_node);

function add_node_type(path: number[], node_type: KeysOfUnion<GameNodeKind>) {
  if (!path) {
    console.error("Tried to add node to undefined node.");
    return;
  }
  const scene_copy = structuredClone(toRaw(scene.value));
  add_to_node(scene_copy.root_node, [...path], create_game_node(node_type));
  update_scene_server(scene_copy);
}

function add_to_node(
  node: GameNodeKind,
  path: number[],
  new_child: GameNodeKind,
) {
  const n = Object.values(node)[0];
  if (path.length === 0) {
    n.children.push(new_child);
    return;
  }
  let p = path.splice(-1);
  add_to_node(n.children[p[0]], path, new_child);
}
</script>
