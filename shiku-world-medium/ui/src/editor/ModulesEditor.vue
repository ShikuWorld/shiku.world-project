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
      v-on:keydown="add_insert_point"
    ></v-text-field>
    <v-chip v-for="insert in module.insert_points"
      >{{ insert.name }}
      <v-btn
        :icon="mdiTrashCan"
        class="modules-editor__point-delete"
        @click="delete_insert_point(insert.name)"
        size="small"
      ></v-btn
    ></v-chip>
    <v-divider></v-divider>
    <v-text-field
      label="Add Output Socket"
      v-model="output_socket_name"
      v-on:keydown="add_output_socket"
    ></v-text-field>
    <v-chip v-for="exit in module.exit_points"
      >{{ exit.name
      }}<v-btn
        :icon="mdiTrashCan"
        class="modules-editor__point-delete"
        @click="delete_exit_point(exit.name)"
        size="small"
      ></v-btn
    ></v-chip>
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
import { ref, toRefs } from "vue";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);

const { save_module_server, delete_module_server } = use_editor_store();
const input_socket_name = ref("");
const output_socket_name = ref("");

function delete_insert_point(insert_point_name: string) {
  console.log(JSON.stringify(module.value.insert_points));
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
