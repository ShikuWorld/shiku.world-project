<template>
  <div class="entities-list">
    <div v-if="scene_node">{{ scene_node }}</div>
    <div v-if="group_node">
      <span>{{ group_node.name }}</span>
      <div v-for="n in group_node.children">
        <SceneNodeList :node="n"></SceneNodeList>
      </div>
    </div>
    <div v-if="physics_node">
      <span>{{ physics_node.name }}</span>
      <div v-for="n in physics_node.children">
        <SceneNodeList :node="n"></SceneNodeList>
      </div>
    </div>
    <div v-if="render_node">
      <span>{{ render_node.name }}</span>
      <div v-for="n in render_node.children">
        <SceneNodeList :node="n"></SceneNodeList>
      </div>
    </div>
  </div>
</template>

<style>
.entities-list {
  padding: 10px;
}
</style>

<script lang="ts" setup>
import {computed, toRefs} from "vue";
import {GameNodeKind} from "@/editor/blueprints/GameNodeKind";
import type {GameNode} from "@/editor/blueprints/GameNode";
import type {Physicality} from "@/editor/blueprints/Physicality";

const props = defineProps<{ node: GameNodeKind }>();
const { node } = toRefs(props);

const scene_node = computed(() => "Scene" in node ? node.Scene as string : undefined);
const group_node = computed(() => "Group" in node ? node.Group as GameNode<string>: undefined);
const physics_node = computed(() => "Physics" in node ? node.Physics as GameNode<Physicality> : undefined);
const render_node = computed(() => "Render" in node ? node.Render as GameNode<Physicality> : undefined);
</script>
