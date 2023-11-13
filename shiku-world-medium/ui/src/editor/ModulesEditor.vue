<template>
  <div>
    <v-text-field
      label="Label"
      :model-value="module.name"
      v-on:change="change_module_name"
    ></v-text-field>
    <v-divider></v-divider>
    <v-text-field
      label="Add Input Socket"
      v-model="input_socket_name"
      v-on:keydown="add_input_socket"
    ></v-text-field>
    <v-chip v-for="insert in module.insert_points">{{ insert.name }}</v-chip>
    <v-divider></v-divider>
    <v-text-field
      label="Add Output Socket"
      v-model="output_socket_name"
      v-on:keydown="add_output_socket"
    ></v-text-field>
    <v-chip v-for="exit in module.exit_points">{{ exit.name }}</v-chip>
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
.modules-editor-delete-button {
  color: red;
}
</style>

<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { use_editor_store } from "@/editor/stores/editor";
import { mdiTrashCan } from "@mdi/js";
import { ref } from "vue";

const { module } = defineProps<{ module: Module }>();
const { save_module_server, delete_module_server } = use_editor_store();
const input_socket_name = ref("");
const output_socket_name = ref("");

function change_module_name(val: Event) {
  save_module_server(module.id, {
    name: (val.target as HTMLInputElement).value,
  });
}

function add_input_socket(event: KeyboardEvent) {
  if (event.key === "Enter") {
    save_module_server(module.id, {
      insert_points: [
        ...module.insert_points,
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
    save_module_server(module.id, {
      exit_points: [
        ...module.exit_points,
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
