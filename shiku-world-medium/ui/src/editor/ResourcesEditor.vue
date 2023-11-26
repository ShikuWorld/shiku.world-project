<template>
  <div>
    <div class="resources-editor--tabs">
      <v-tabs v-model="selected_resource_tab" bg-color="primary">
        <v-tab value="main">Main</v-tab>
        <v-tab
          v-for="resource in open_resources"
          :key="resource.path"
          :value="resource.path"
          >{{ resource.file_name
          }}<v-icon
            :icon="mdiTrashCan"
            @click="close_resource(resource.path)"
          ></v-icon
        ></v-tab>
      </v-tabs>
    </div>
    <v-window v-model="selected_resource_tab">
      <v-window-item value="main">
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
        <v-dialog v-model="create_tileset_dialog" width="800">
          <CreateTileset
            @close="create_tileset_dialog = false"
            @save="save_tileset"
          ></CreateTileset>
        </v-dialog>
      </v-window-item>
      <v-window-item
        v-for="resource in open_resources"
        :key="resource.path"
        :value="resource.path"
        >{{ resource.kind }}</v-window-item
      >
    </v-window>
  </div>
</template>
<script lang="ts" setup>
import { mdiPlus, mdiTrashCan } from "@mdi/js";
import CreateTileset from "@/editor/editor/CreateTileset.vue";
import { computed, ref } from "vue";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";
import { Resource } from "@/editor/blueprints/Resource";
const create_tileset_dialog = ref(false);
const { create_tileset_server, close_resource } = use_editor_store();
const { open_resource_paths, modules, selected_resource_tab } = storeToRefs(
  use_editor_store(),
);
const available_resources = computed(
  () =>
    new Map(
      Object.values(modules.value).flatMap((m) =>
        m.resources.map((resource) => [resource.path, resource]),
      ),
    ),
);
const open_resources = computed(
  () =>
    open_resource_paths.value.map((path) =>
      available_resources.value.get(path),
    ) as Resource[],
);
function save_tileset(tileset: Tileset) {
  create_tileset_dialog.value = false;
  create_tileset_server(tileset);
}
</script>
<style></style>
