<template>
  <div class="add-resources-modal">
    <v-text-field v-model="current_path"></v-text-field>
    <v-list density="compact">
      <v-list-item
        class="add-resources-modal__entry add-resources-modal__folder"
        color="primary"
        v-if="current_path != ''"
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
        <template v-slot:prepend> <v-icon :icon="mdiFolder"></v-icon> </template
        ><v-list-item-title v-text="dir"></v-list-item-title>
      </v-list-item>
      <v-list-item
        class="add-resources-modal__entry add-resources-modal__resource"
        v-for="resource in current_file_browser_result.resources"
        @click="toggle_resource_on_module(resource)"
        :active="module.resources.find((r) => r.path === resource.path)"
        :class="{
          'add-resources-modal__entry--active': module.resources.find(
            (r) => r.path === resource.path,
          ),
        }"
        :key="resource.path"
      >
        <template v-slot:prepend>
          <v-icon :icon="mdiPackageVariant"></v-icon> </template
        ><v-list-item-title v-text="resource.file_name"></v-list-item-title>
      </v-list-item>
    </v-list>
  </div>
</template>

<style>
.add-resources-modal__entry--active {
  background-color: blue;
}
.add-resources-modal {
  background-color: #37474f;
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
import { ref, toRefs } from "vue";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";
import { mdiFolder, mdiPackageVariant } from "@mdi/js";
import { Resource } from "@/editor/blueprints/Resource";

const current_path = ref("");
const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);
const { browse_folder, save_module_server } = use_editor_store();
const { current_file_browser_result } = storeToRefs(use_editor_store());

function toggle_resource_on_module(resource: Resource) {
  const resource_in_module = module.value.resources.find(
    (r) => r.path === resource.path,
  );
  if (resource_in_module) {
    save_module_server(module.value.id, {
      resources: module.value.resources.filter((r) => r.path !== resource.path),
    });
  } else {
    save_module_server(module.value.id, {
      resources: [...module.value.resources, resource],
    });
  }
}
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
