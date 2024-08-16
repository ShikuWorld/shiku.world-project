<template>
  <v-card class="settings-editor">
    <v-switch
      label="Main door open"
      :hide-details="true"
      density="compact"
      color="secondary"
      :model-value="main_door_status"
      @click="update_main_door_status(!main_door_status)"
    ></v-switch>
    <v-switch
      label="Back door open"
      :hide-details="true"
      density="compact"
      :model-value="back_door_status"
      @click="update_back_door_status(!back_door_status)"
    ></v-switch>
    <v-btn @click="toggle_entity_colliders">Toggle entity colliders</v-btn>
    <v-btn @click="toggle_terrain_collisions">Toggle terrain collision</v-btn>
  </v-card>
</template>

<script setup lang="ts">
import { use_config_store } from "@/editor/stores/config";
import { storeToRefs } from "pinia";
import { use_game_instances_store } from "@/editor/stores/game-instances";
import { use_medium_api } from "@/editor/api";

const { toggle_entity_colliders } = use_game_instances_store();
const { toggle_terrain_collisions } = use_medium_api();
const { main_door_status, back_door_status } = storeToRefs(use_config_store());
const { set_main_door_status_server, set_back_door_status_server } =
  use_config_store();

const update_main_door_status = (new_value: boolean | null) => {
  if (new_value !== null) {
    set_main_door_status_server(new_value);
  }
};
const update_back_door_status = (new_value: boolean | null) => {
  if (new_value !== null) {
    set_back_door_status_server(new_value);
  }
};
</script>

<style scoped>
.settings-editor {
  pointer-events: all;
  padding: 16px;
  margin: 16px;
}
</style>
