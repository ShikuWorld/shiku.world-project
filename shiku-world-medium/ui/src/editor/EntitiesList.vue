<template>
  <div class="entities-list">
    <div v-if="module.maps.length === 0">
      No maps yet, <a @click="create_map">click</a> to create.
    </div>
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

function create_map() {}
</script>
