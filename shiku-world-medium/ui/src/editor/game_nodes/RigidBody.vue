<template>
  <div>{{ game_node.data.body }}</div>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { GameNode } from "@/editor/blueprints/GameNode";
import { RigidBody } from "@/editor/blueprints/RigidBody";

const props = defineProps<{ game_node: GameNode<RigidBody> }>();
const { game_node } = toRefs(props);
</script>
<style></style>
