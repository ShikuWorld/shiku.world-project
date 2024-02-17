<template>
  <div class="entities-list">
    <div v-if="instance_node">{{ instance_node }}</div>
    <div v-if="container_node">
      <span>{{ container_node.name }}</span>
      <div v-for="n in container_node.children">
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
import {Render} from "@/editor/blueprints/Render";

const props = defineProps<{ node: GameNodeKind }>();
const { node } = toRefs(props);

const instance_node = computed(() => "Instance" in node ? node.Instance as string : undefined);
const container_node = computed(() => "Container" in node ? node.Container as GameNode<string>: undefined);
const physics_node = computed(() => "Physics" in node ? node.Physics as GameNode<Physicality> : undefined);
const render_node = computed(() => "Render" in node ? node.Render as GameNode<Render> : undefined);
</script>
