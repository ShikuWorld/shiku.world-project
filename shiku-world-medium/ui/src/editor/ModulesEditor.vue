<template>
  <div class="modules-editor-inner">
    <v-text-field
      label="Label"
      :model-value="module.name"
      v-on:change="change_module_name"
      density="compact"
      hide-details="auto"
    ></v-text-field>
    <v-divider></v-divider>
    <v-text-field
      label="Insert Point"
      v-model="input_socket_name"
      v-on:keydown="add_insert_point"
      density="compact"
      hide-details="auto"
    ></v-text-field>
    <ModuleSlots :slots="module.insert_points" @delete="delete_insert_point" />
    <v-divider></v-divider>
    <v-text-field
      label="Exit Point"
      v-model="output_socket_name"
      v-on:keydown="add_output_socket"
      density="compact"
      hide-details="auto"
    ></v-text-field>
    <ModuleSlots :slots="module.exit_points" @delete="delete_exit_point" />
    <v-divider></v-divider>
    <v-btn @click="set_main_module_to_edit(module.id)">Select as Main</v-btn>
    <v-divider></v-divider>

    <v-dialog width="800">
      <template v-slot:activator="{ props }">
        <v-btn v-bind="props">Assign resources</v-btn>
      </template>

      <template v-slot:default="{ isActive }">
        <AddResourcesModal :module="module"></AddResourcesModal>
      </template>
    </v-dialog>

    <v-divider></v-divider>

    <v-dialog width="500">
      <template v-slot:activator="{ props }">
        <v-btn
          :icon="mdiTrashCan"
          class="modules-editor-delete-button"
          size="small"
          v-bind="props"
        ></v-btn>
      </template>

      <template v-slot:default="{ isActive }">
        <v-card :title="`Delete ${module.name}?`">
          <v-card-actions>
            <v-spacer></v-spacer>

            <v-btn
              text="Yes!"
              @click="
                delete_module(module.id);
                isActive.value = false;
              "
            ></v-btn>
          </v-card-actions>
        </v-card>
      </template>
    </v-dialog>
  </div>
</template>

<style>
.modules-editor-inner {
  padding: 10px;
}
.modules-editor-delete-button {
  color: red;
  margin: 10px 0;
}
</style>

<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { use_editor_store } from "@/editor/stores/editor";
import { mdiTrashCan } from "@mdi/js";
import { ref, toRefs } from "vue";
import ModuleSlots from "@/editor/editor/ModuleSlots.vue";
import AddResourcesModal from "@/editor/editor/AddResourcesModal.vue";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);

const { save_module_server, delete_module_server, set_main_module_to_edit } =
  use_editor_store();
const input_socket_name = ref("");
const output_socket_name = ref("");

function delete_insert_point(insert_point_name: string) {
  save_module_server(module.value.id, {
    insert_points: module.value.insert_points.filter(
      (p) => p.name !== insert_point_name,
    ),
  });
}

function delete_exit_point(exit_point_name: string) {
  save_module_server(module.value.id, {
    exit_points: module.value.exit_points.filter(
      (p) => p.name !== exit_point_name,
    ),
  });
}

function change_module_name(val: Event) {
  save_module_server(module.value.id, {
    name: (val.target as HTMLInputElement).value,
  });
}

function add_insert_point(event: KeyboardEvent) {
  if (event.key === "Enter") {
    save_module_server(module.value.id, {
      insert_points: [
        ...module.value.insert_points,
        {
          name: input_socket_name.value,
          condition_script: "",
        },
      ],
    });
    input_socket_name.value = "";
  }
}

function add_output_socket(event: KeyboardEvent) {
  if (event.key === "Enter") {
    save_module_server(module.value.id, {
      exit_points: [
        ...module.value.exit_points,
        {
          name: output_socket_name.value,
          condition_script: "",
        },
      ],
    });
    output_socket_name.value = "";
  }
}

function delete_module(id: string) {
  delete_module_server(id);
}
</script>
