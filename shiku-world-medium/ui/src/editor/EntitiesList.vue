<template>
  <div class="entities-list">
    <div v-if="module.resources.length === 0"></div>
  </div>
</template>

<style>
.entities-list {
  padding: 10px;
}
</style>

<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { toRefs } from "vue";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);
</script>
