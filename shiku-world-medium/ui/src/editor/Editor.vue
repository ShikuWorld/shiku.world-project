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
        <v-expansion-panel title="Scene" v-if="current_main_scene">
          <v-expansion-panel-text
            ><scene-editor :scene="current_main_scene"></scene-editor
          ></v-expansion-panel-text>
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
            :module_instances="module_instance_map[selected_module.id] || []"
            @world_click="select_as_main_instance"
          />
        </v-expansion-panel>
        <v-btn @click="load_map_palette">Map palette</v-btn>
      </v-expansion-panels>
    </div>
    <div class="editor-main-view">
      <v-window v-model="tab">
        <v-window-item value="current"> </v-window-item>
        <v-window-item value="modules">
          <ModulesGraph class="modules-editor"></ModulesGraph>
        </v-window-item>
        <v-window-item value="resources">
          <ResourcesEditor class="resources-editor"></ResourcesEditor>
        </v-window-item>
      </v-window>
    </div>
    <div class="editor-nav-right">
      <div v-if="active_component === 'nothing'">Edit something</div>
      <div v-if="active_component === 'game_node' && selected_node">
        <GameNodeInspector :node="selected_node"></GameNodeInspector>
      </div>
      <div v-if="active_component === 'module'">
        <ModulesEditor
          v-if="selected_module"
          :module="selected_module"
          :module_instances="module_instance_map[selected_module.id] || []"
        ></ModulesEditor>
      </div>
      <div v-if="active_component === 'map'">
        <TileSelector :tilesets="tilesets_of_current_module"></TileSelector>
      </div>
      <div v-if="active_component === 'tile'">
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
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import ModulesEditor from "@/editor/editor/ModulesEditor.vue";
import { resource_key, use_editor_store } from "@/editor/stores/editor";
import ResourcesEditor from "@/editor/editor/ResourcesEditor.vue";
import ModuleResourceList from "@/editor/editor/ModuleResourceList.vue";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import TileEditor from "@/editor/editor/TileEditor.vue";
import ModuleInstanceList from "@/editor/editor/ModuleInstanceList.vue";
import { GameMap } from "@/editor/blueprints/GameMap";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import TileSelector from "@/editor/editor/TileSelector.vue";
import { use_inspector_store } from "@/editor/stores/inspector";
import GameNodeInspector from "@/editor/editor/GameNodeInspector.vue";
import { use_resources_store } from "@/editor/stores/resources";
import SceneEditor from "@/editor/editor/SceneEditor.vue";
import { Scene } from "@/editor/blueprints/Scene";

const tab = ref<number>(0);
const {
  selected_module_id,
  selected_tileset_path,
  selected_tile_id,
  module_instance_map,
  current_main_instance,
  selected_tile_position,
  tile_brush,
} = storeToRefs(use_editor_store());
const {
  add_open_resource_path,
  set_selected_resource_tab,
  game_instance_exists,
  set_current_main_instance,
} = use_editor_store();

const { game_map_map, tileset_map } = storeToRefs(use_resources_store());
const {
  get_module,
  get_tileset,
  load_modules,
  update_map_server,
  get_resource_server,
  get_scene,
} = use_resources_store();

const { active_component, component_stores } = storeToRefs(
  use_inspector_store(),
);
const { set_inspector_component } = use_inspector_store();
const selected_node = computed(
  () => component_stores.value.game_node.selected_game_node,
);

load_modules();

const selected_module = computed(() => get_module(selected_module_id?.value));
const selected_tileset = computed(() =>
  get_tileset(selected_tileset_path.value),
);
const tilesets_of_current_module = computed(() => {
  return selected_module.value.resources
    .filter((r) => r.kind === "Tileset")
    .map((r) => {
      return tileset_map.value[r.path];
    });
});
function load_map_palette() {
  if (selected_module.value) {
    const tilesets_to_load: BlueprintResource[] = [];
    for (const r of selected_module.value.resources) {
      if (r.kind === "Tileset" && !tileset_map.value[r.path]) {
        tilesets_to_load.push(r);
      }
    }
    if (tilesets_to_load.length > 0) {
      for (const r of tilesets_to_load) {
        console.log("loading", r);
        get_resource_server(r.path);
      }
    }
  }
  set_inspector_component("map");
}
const current_main_map = computed<GameMap | undefined>(() => {
  if (current_main_instance.value?.world_id && game_map_map.value) {
    return Object.values(game_map_map.value).find(
      (m) => m.world_id === current_main_instance.value.world_id,
    );
  }
  return undefined;
});

const current_main_scene = computed<Scene | undefined>(() => {
  if (current_main_map.value && current_main_map.value.main_scene) {
    return get_scene(current_main_map.value.main_scene);
  }
  return undefined;
});

watch(selected_tile_position, () =>
  on_tile_click(
    "Terrain",
    selected_tile_position.value.x,
    selected_tile_position.value.y + 1,
  ),
);

function on_tile_click(layer_kind: LayerKind, tile_x: number, tile_y: number) {
  if (current_main_map.value) {
    let game_map = current_main_map.value;
    for (const chunk_id of fill_map(
      game_map,
      layer_kind,
      tile_x,
      tile_y,
      tile_brush.value,
    ).values()) {
      const updated_chunk = game_map.terrain[layer_kind][chunk_id];
      update_map_server({
        name: game_map.name,
        resource_path: game_map.resource_path,
        chunk: [layer_kind, updated_chunk],
        scene: null,
      });
    }
  }
}

function fill_map(
  game_map: GameMap,
  layer_kind: LayerKind,
  start_x: number,
  start_y: number,
  brush: number[][],
): Set<number> {
  const filled_chunks = new Set<number>();
  for (let y = 0; y < brush.length; y++) {
    for (let x = 0; x < brush[0].length; x++) {
      let chunk_x = Math.floor((start_x + x) / game_map.chunk_size);
      let chunk_y = Math.floor((start_y + y) / game_map.chunk_size);
      if (!game_map.terrain[layer_kind][cantorPair(chunk_x, chunk_y)]) {
        game_map.terrain[layer_kind][cantorPair(chunk_x, chunk_y)] = {
          position: [chunk_x, chunk_y],
          data: new Array(game_map.chunk_size * game_map.chunk_size).fill(0),
        };
      }
      const chunk = game_map.terrain[layer_kind][cantorPair(chunk_x, chunk_y)];
      filled_chunks.add(cantorPair(chunk_x, chunk_y));
      let chunk_tile_x = start_x + x - chunk_x * game_map.chunk_size;
      let chunk_tile_y = start_y + y - chunk_y * game_map.chunk_size;

      chunk.data[chunk_tile_y * game_map.chunk_size + chunk_tile_x] =
        brush[y][x];
    }
  }
  return filled_chunks;
}

function toNatural(num: number): number {
  if (num < 0) {
    return -2 * num - 1;
  } else {
    return 2 * num;
  }
}

function cantorPair(x: number, y: number): number {
  const xx = toNatural(x);
  const yy = toNatural(y);
  return ((xx + yy) * (xx + yy + 1)) / 2 + yy;
}
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
