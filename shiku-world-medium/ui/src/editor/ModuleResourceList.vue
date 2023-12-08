<template>
  <v-list density="compact">
    <v-list-item
      density="compact"
      height="20"
      class="add-resources-modal__current-resources-item add-resources-modal__entry add-resources-modal__resource"
      v-for="resource in module.resources"
      :prepend-icon="mdiPackageVariant"
      :key="resource.path"
      @click="emit('resourceClick', resource)"
    >
      <template v-if="showRemove" v-slot:append>
        <v-icon
          @click="toggle_resource_on_module(module.id, resource)"
          :icon="mdiTrashCan"
        ></v-icon>
      </template>
      <v-list-item-title v-text="resource.file_name"></v-list-item-title>
      <v-list-item-subtitle v-text="resource.dir"></v-list-item-subtitle>
    </v-list-item>
  </v-list>
</template>
<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { toRefs } from "vue";
import { mdiPackageVariant, mdiTrashCan } from "@mdi/js";
import { use_editor_store } from "@/editor/stores/editor";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
const { toggle_resource_on_module } = use_editor_store();
const emit = defineEmits<{
  (e: "resourceClick", value: BlueprintResource): void;
}>();
const props = defineProps<{ module: Module; showRemove?: boolean }>();
const { module, showRemove } = toRefs(props);
</script>
<style>
.add-resources-modal__entry {
  cursor: pointer;
}

.add-resources-modal__entry:hover {
  background-color: #37474f;
}
</style>
