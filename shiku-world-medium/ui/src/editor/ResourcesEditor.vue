<template>
  <div>
    <v-btn
      :icon="mdiPlus"
      id="menu-activator"
      density="comfortable"
      color="primary"
      size="small"
    >
    </v-btn>
    <v-menu activator="#menu-activator">
      <v-list>
        <v-list-item>
          <v-list-item-title @click="create_tileset_dialog = true"
            >Tileset</v-list-item-title
          >
        </v-list-item>
      </v-list>
    </v-menu>
    <v-dialog v-model="create_tileset_dialog" width="auto">
      <CreateTileset
        @close="create_tileset_dialog = false"
        @save="save_tileset"
      ></CreateTileset>
    </v-dialog>
  </div>
</template>
<script lang="ts" setup>
import { mdiPlus } from "@mdi/js";
import CreateTileset from "@/editor/editor/CreateTileset.vue";
import { ref } from "vue";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
const create_tileset_dialog = ref(false);
function save_tileset(tileset: Tileset) {
  create_tileset_dialog.value = false;
  console.log(tileset);
}
</script>
<style></style>
