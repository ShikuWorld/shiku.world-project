<template>
  <div class="entities-list">
    <div v-if="is_node_type(scene.root_node, 'scene')">Scene</div>
    <div v-if="is_node_type(scene.root_node, 'group')">Group</div>
    <div v-if="is_node_type(scene.root_node, 'physics')">Physics</div>
    <div v-if="is_node_type(scene.root_node, 'render')">Render</div>
  </div>
</template>

<style>
.entities-list {
  padding: 10px;
}
</style>

<script lang="ts" setup>
import { toRefs } from "vue";
import type {Scene} from "@/editor/blueprints/Scene";
import {GameNodeKind} from "@/editor/blueprints/GameNodeKind";
import {match, P} from "ts-pattern";

const props = defineProps<{ scene: Scene }>();
const { scene } = toRefs(props);
type SceneNodeType = "scene" | "physics" | "group" | "render";
function is_node_type(node: GameNodeKind, node_type: SceneNodeType): boolean {
  return match(node)
      .with({Scene: P.select()}, (): SceneNodeType => 'scene')
      .with({Physics: P.select()}, (): SceneNodeType => 'physics')
      .with({Group: P.select()}, (): SceneNodeType => 'group')
      .with({Render: P.select()}, (): SceneNodeType => 'render')
      .exhaustive() === node_type;
}
</script>
