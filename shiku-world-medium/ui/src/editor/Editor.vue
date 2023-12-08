<template>
  <div class="editor-wrapper">
    <div class="editor-nav-top">
      <v-tabs v-model="tab" bg-color="primary">
        <v-tab value="current">Current</v-tab>
        <v-tab value="modules">Modules</v-tab>
        <v-tab value="resources">Resources</v-tab>
      </v-tabs>
    </div>
    <div class="editor-nav-left">
      <v-expansion-panels
        :multiple="true"
        variant="accordion"
        v-if="selected_module"
      >
        <v-expansion-panel title="Maps">
          <v-expansion-panel-text> </v-expansion-panel-text>
        </v-expansion-panel>
        <v-expansion-panel title="Resources">
          <v-expansion-panel-text>
            <ModuleResourceList
              @resourceClick="open_resource_editor"
              :module="selected_module"
            />
          </v-expansion-panel-text>
        </v-expansion-panel>
        <v-expansion-panel title="Instances">
          <ModuleInstanceList
            :module="selected_module"
            :show_current_instance="true"
            :module_instances="module_instance_map[selected_module.id]"
            @world_click="select_as_main_instance"
          />
        </v-expansion-panel>
      </v-expansion-panels>
    </div>
    <div class="editor-main-view">
      <v-window v-model="tab">
        <v-window-item value="current">
          <MapEditor
            class="map-editor"
            v-if="current_main_map"
            :map="current_main_map"
          ></MapEditor>
        </v-window-item>
        <v-window-item value="modules">
          <ModulesGraph class="modules-editor"></ModulesGraph>
        </v-window-item>
        <v-window-item value="resources">
          <ResourcesEditor class="resources-editor"></ResourcesEditor>
        </v-window-item>
      </v-window>
    </div>
    <div class="editor-nav-right">
      <div v-if="side_bar_editor === 'nothing'">Edit something</div>
      <div v-if="side_bar_editor === 'module'">
        <ModulesEditor
          v-if="selected_module"
          :module="selected_module"
          :module_instances="module_instance_map[selected_module.id]"
        ></ModulesEditor>
      </div>
      <div v-if="side_bar_editor === 'tile'">
        <TileEditor
          v-if="selected_tileset && selected_tile_id"
          :tileset="selected_tileset"
          :tile_id="selected_tile_id"
        ></TileEditor>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import ModulesGraph from "@/editor/editor/ModulesGraph.vue";
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import ModulesEditor from "@/editor/editor/ModulesEditor.vue";
import { resource_key, use_editor_store } from "@/editor/stores/editor";
import ResourcesEditor from "@/editor/editor/ResourcesEditor.vue";
import ModuleResourceList from "@/editor/editor/ModuleResourceList.vue";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import TileEditor from "@/editor/editor/TileEditor.vue";
import ModuleInstanceList from "@/editor/editor/ModuleInstanceList.vue";
import MapEditor from "@/editor/editor/MapEditor.vue";
import { GameMap } from "@/editor/blueprints/GameMap";

const tab = ref<number>(0);
const {
  selected_module_id,
  selected_tileset_path,
  selected_tile_id,
  side_bar_editor,
  module_instance_map,
  current_main_instance,
  game_map_map,
} = storeToRefs(use_editor_store());
const {
  get_module,
  get_tileset,
  load_modules,
  add_open_resource_path,
  set_selected_resource_tab,
  game_instance_exists,
  set_current_main_instance,
} = use_editor_store();
load_modules();

const selected_module = computed(() => get_module(selected_module_id?.value));
const selected_tileset = computed(() =>
  get_tileset(selected_tileset_path.value),
);
const current_main_map = computed<GameMap | undefined>(() => {
  if (current_main_instance.value?.world_id && game_map_map.value) {
    return Object.values(game_map_map.value).find(
      (m) => m.world_id === current_main_instance.value.world_id,
    );
  }
  return undefined;
});
function select_as_main_instance(
  _module_id: string,
  instance_id: string,
  world_id: string,
) {
  if (game_instance_exists(instance_id, world_id)) {
    set_current_main_instance(instance_id, world_id);
  }
}
function open_resource_editor(resource: BlueprintResource) {
  tab.value = 2;
  let path_index = add_open_resource_path(resource_key(resource));
  setTimeout(() => {
    set_selected_resource_tab(path_index + 1);
  }, 50);
}
</script>

<style lang="scss">
.editor-wrapper {
  display: flex;
  flex-wrap: wrap;
}

.map-editor {
  height: 100vh;
  pointer-events: all;
}

.modules-editor {
  height: 100vh;
  pointer-events: all;
}

.resources-editor {
  pointer-events: all;
  height: 100%;
}

.editor-main-view {
  display: inline-block;
  flex-grow: 1;
}

.editor-nav-top {
  width: 100%;
  display: block;
}

.editor-nav-left,
.editor-nav-right {
  width: 200px;
  height: 100vh;
  background-color: rgb(var(--v-theme-primary));
}

.editor-nav-top,
.editor-nav-left,
.editor-nav-right {
  pointer-events: all;
}
</style>
