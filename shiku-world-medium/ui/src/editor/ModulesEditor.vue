<template>
  <div>
    <v-text-field
      label="Label"
      :model-value="module.name"
      v-on:change="change_module_name"
    ></v-text-field>
  </div>
</template>

<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { use_editor_store } from "@/editor/stores/editor";

const { module } = defineProps<{ module: Module }>();
const { update_module } = use_editor_store();

function change_module_name(val: Event) {
  update_module({
    id: module.id,
    name: (val.target as HTMLInputElement).value,
  });
}
</script>
