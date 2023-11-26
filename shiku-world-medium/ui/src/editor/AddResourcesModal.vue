<template>
  <div class="add-resources-modal">
    <div class="add-resources-modal__title">{{ module.name }}</div>
    <div class="add-resources-modal__file-browser">
      <v-text-field
        density="compact"
        :hide-details="true"
        class="add-resources-modal__file-browser-input"
        v-model="current_path"
      ></v-text-field>
      <v-list class="add-resources-modal__file-browser-items" density="compact">
        <v-list-item
          class="add-resources-modal__entry add-resources-modal__folder"
          color="primary"
          :disabled="current_path == ''"
          @click="go_up()"
        >
          <v-list-item-title v-text="'...'"></v-list-item-title>
        </v-list-item>
        <v-list-item
          class="add-resources-modal__entry add-resources-modal__folder"
          color="primary"
          v-for="dir in current_file_browser_result.dirs"
          :key="dir"
          @click="browse(dir)"
        >
          <template v-slot:prepend>
            <v-icon :icon="mdiFolder"></v-icon> </template
          ><v-list-item-title v-text="dir"></v-list-item-title>
        </v-list-item>
        <v-list-item
          class="add-resources-modal__entry add-resources-modal__resource"
          v-for="resource in current_file_browser_result.resources"
          @click="toggle_resource_on_module(module.id, resource)"
          :active="active_resources_set.has(resource.path)"
          :key="resource.path"
        >
          <template v-slot:prepend>
            <v-icon :icon="mdiPackageVariant"></v-icon> </template
          ><v-list-item-title v-text="resource.file_name"></v-list-item-title>
        </v-list-item>
      </v-list>
    </div>
    <div class="add-resources-modal__current-resources">
      <ModuleResourceList :show-remove="true" :module="module" />
    </div>
  </div>
</template>

<style>
.add-resources-modal {
  background-color: #37474f;
  display: flex;
  flex-wrap: wrap;
  align-content: baseline;
  width: 100%;
}
.add-resources-modal__title {
  display: inline-block;
  width: 100%;
  padding: 10px;
}
.add-resources-modal__file-browser-input {
  flex-grow: 0;
}
.add-resources-modal__file-browser-items {
  flex-grow: 1;
  overflow: hidden !important;
  overflow-y: scroll !important;
}

.add-resources-modal__file-browser {
  display: flex;
  flex-direction: column;
  width: 66.666%;
  height: 300px;
  max-height: 300px;
}
.add-resources-modal__current-resources {
  width: 33.333%;
  display: flex;
  flex-direction: column;
  max-height: 300px;
  overflow: hidden;
  overflow-y: scroll;
  height: 300px;
}
.add-resources-modal__entry {
  cursor: pointer;
}
.add-resources-modal__entry:hover {
  background-color: #37474f;
}
</style>

<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { computed, ref, toRefs } from "vue";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";
import { mdiFolder, mdiPackageVariant } from "@mdi/js";
import ModuleResourceList from "@/editor/editor/ModuleResourceList.vue";

const current_path = ref("");
const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);
const active_resources_set = computed(() =>
  module.value ? new Set(module.value.resources.map((r) => r.path)) : new Set(),
);
const { browse_folder, toggle_resource_on_module } = use_editor_store();
const { current_file_browser_result } = storeToRefs(use_editor_store());

function go_up() {
  if (current_path.value.includes("/")) {
    const split = current_path.value.split("/");
    split.pop();
    current_path.value = split.join("/");
  } else {
    current_path.value = "";
  }
  browse_folder(current_path.value);
}

function browse(dir: string) {
  current_path.value = `${
    current_path.value === "" ? "" : current_path.value + "/"
  }${dir}`;
  browse_folder(current_path.value);
}

browse_folder(current_path.value);
</script>
