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
          <v-list-item-title @click="add_node_type(node_type.value)">{{
            node_type.label
          }}</v-list-item-title>
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
import { toRefs } from "vue";
import type { Scene } from "@/editor/blueprints/Scene";
import SceneNodeList from "@/editor/editor/SceneNodeList.vue";
import { mdiPlus } from "@mdi/js";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { match } from "ts-pattern";
import { GameNode } from "@/editor/blueprints/GameNode";
import { v4 as uuidv4 } from "uuid";
import { Physicality } from "@/editor/blueprints/Physicality";
import { Render } from "@/editor/blueprints/Render";
import { RenderKind } from "@/editor/blueprints/RenderKind";
type KeysOfUnion<T> = T extends T ? keyof T : never;
const node_type_options: { value: KeysOfUnion<GameNodeKind>; label: string }[] =
  [
    { value: "Instance", label: "Instance" },
    { value: "Render", label: "Render" },
    { value: "Physics", label: "Physics" },
    { value: "Container", label: "Container" },
  ];
const props = defineProps<{ scene: Scene }>();
const { scene } = toRefs(props);
function add_node_type(
  node: GameNode<unknown>,
  value: KeysOfUnion<GameNodeKind>,
) {
  node.children.push(
    match(value)
      .with(
        "Container",
        (): GameNodeKind => ({
          Container: {
            name: "Container",
            id: uuidv4(),
            data: "",
            script: null,
            children: [],
          },
        }),
      )
      .with(
        "Physics",
        (): GameNodeKind => ({
          Physics: {
            name: "Physics",
            id: uuidv4(),
            data: {
              position: [0, 0],
              velocity: [0, 0],
              rotation: 0,
              body: "None",
            },
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
      .with("Instance", (): GameNodeKind => ({ Instance: "" }))
      .exhaustive(),
  );
}
</script>
