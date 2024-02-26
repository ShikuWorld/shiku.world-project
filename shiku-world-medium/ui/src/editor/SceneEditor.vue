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
            @click="add_node_type(selected_node, node_type.value)"
            >{{ node_type.label }}</v-list-item-title
          >
        </v-list-item>
      </v-list>
    </v-menu>
    <scene-node-list :node="scene.root_node"></scene-node-list>
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
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { match } from "ts-pattern";
import { v4 as uuidv4 } from "uuid";
import { KeysOfUnion } from "@/editor/utils";
import { storeToRefs } from "pinia";
import { use_inspector_store } from "@/editor/stores/inspector";

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
const selected_node = computed(
  () => component_stores.value.game_node.selected_game_node,
);

function add_node_type(node: GameNodeKind, value: KeysOfUnion<GameNodeKind>) {
  const game_node = Object.values(node)[0];
  if (!node) {
    console.error("Tried to add node to undefined node.");
    return;
  }
  if (value === "Instance") {
    console.error("Cannot add instances here");
    return;
  }
  game_node.children.push(
    match(value)
      .with(
        "RigidBody",
        (): GameNodeKind => ({
          RigidBody: {
            name: "RigidBody",
            id: uuidv4(),
            data: {
              position: [0, 0],
              velocity: [0, 0],
              rotation: 0,
              body: "Dynamic",
            },
            script: null,
            children: [],
          },
        }),
      )
      .with(
        "Collider",
        (): GameNodeKind => ({
          Collider: {
            name: "Collider",
            id: uuidv4(),
            data: { kind: "Solid", shape: { Ball: 0 } },
            script: null,
            children: [],
          },
        }),
      )
      .with(
        "Node",
        (): GameNodeKind => ({
          Node: {
            name: "Node",
            id: uuidv4(),
            data: "",
            script: null,
            children: [],
          },
        }),
      )
      .with(
        "Render",
        (): GameNodeKind => ({
          Render: {
            name: "Render",
            id: uuidv4(),
            data: { offset: [0, 0], layer: "BG00", kind: "Sprite" },
            script: null,
            children: [],
          },
        }),
      )
      .exhaustive(),
  );
}
</script>
