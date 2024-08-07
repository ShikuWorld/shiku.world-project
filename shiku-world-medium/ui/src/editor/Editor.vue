<template>
  <div class="editor-wrapper">
    <div class="editor-nav-top">
      <v-tabs
        :model-value="selected_nav_top_tab"
        @update:model-value="
          (v) =>
            set_selected_nav_top_tab(v as EditorStore['selected_nav_top_tab'])
        "
        bg-color="primary"
      >
        <v-tab value="current">Current</v-tab>
        <v-tab value="modules">Modules</v-tab>
        <v-tab value="resources">Resources</v-tab>
      </v-tabs>
    </div>
    <RhaiEditor
      v-if="selected_script_resource_path"
      :script_resource_path="selected_script_resource_path"
      ref="rhai_editor"
    ></RhaiEditor>
    <div class="editor-nav-left">
      <v-expansion-panels
        :multiple="true"
        variant="accordion"
        v-if="selected_module"
      >
        <v-expansion-panel v-if="selected_scene">
          <v-expansion-panel-title
            @contextmenu="on_selected_scene_context_menu($event)"
          >
            {{ selected_scene.name }}
          </v-expansion-panel-title>
          <v-expansion-panel-text>
            <SceneEditor
              :scene="selected_scene"
              :is_scene_instance="false"
              :menu_id="'create-scene-node'"
              @remove_node="on_remove_node_from_scene"
              @edit_script="on_edit_script"
              @add_node="on_add_node_to_scene"
            ></SceneEditor>
          </v-expansion-panel-text>
        </v-expansion-panel>
        <v-expansion-panel title="World" v-if="current_main_instance_scene">
          <v-expansion-panel-text>
            <SceneEditor
              :scene="current_main_instance_scene"
              :is_scene_instance="true"
              :menu_id="'create-instance-node'"
              @remove_node="on_remove_node_from_scene"
              @edit_script="on_edit_script"
              @add_node="on_add_node_to_scene"
            ></SceneEditor>
          </v-expansion-panel-text>
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
        <v-btn @click="reset_selected_world">Reset selected world</v-btn>
        <v-btn @click="load_map_palette">Map palette</v-btn>
      </v-expansion-panels>
    </div>
    <div class="editor-main-view">
      <v-window v-model="selected_nav_top_tab">
        <v-window-item value="current"></v-window-item>
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
      <div
        v-if="
          active_component === 'game_node' &&
          selected_node_resource_path &&
          selected_node &&
          selected_game_node &&
          selected_game_node_path
        "
      >
        <GameNodeInspector
          :node="selected_node"
          :scene_resource_path="selected_node_resource_path"
          :path="selected_game_node_path"
          :key="selected_game_node.id"
          :is_instance="selected_node_is_instance"
        ></GameNodeInspector>
      </div>
      <div v-if="active_component === 'module'">
        <ModulesEditor
          v-if="selected_module"
          :module="selected_module"
          :module_instances="module_instance_map[selected_module.id] || []"
        ></ModulesEditor>
      </div>
      <div v-if="active_component === 'map' && selected_module">
        <TileSelector
          :tilesets="tilesets_of_current_module"
          :gid_map="selected_module.gid_map"
        ></TileSelector>
      </div>
      <div v-if="active_component === 'tile'">
        <TileEditor
          v-if="selected_tileset && selected_tile_id != undefined"
          :tileset="selected_tileset"
          :tile_id="selected_tile_id"
        ></TileEditor>
      </div>
      <div v-if="active_component === 'character_animation_state'">
        <CharacterAnimationNodeInspector
          v-if="selected_character_animation"
          :character_animation="selected_character_animation"
          :selected_state_id="
            component_stores.character_animation.selected_state_id
          "
        ></CharacterAnimationNodeInspector>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import ModulesGraph from "@/editor/editor/ModulesGraph.vue";
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import ModulesEditor from "@/editor/editor/ModulesEditor.vue";
import {
  EditorStore,
  resource_key,
  use_editor_store,
} from "@/editor/stores/editor";
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
import {
  get_generic_game_node,
  get_node_by_path,
  scene_key,
  use_resources_store,
} from "@/editor/stores/resources";
import SceneEditor from "@/editor/editor/SceneEditor.vue";
import { use_game_instances_store } from "@/editor/stores/game-instances";
import ContextMenu from "@imengyu/vue3-context-menu";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { Entity } from "@/editor/blueprints/Entity";
import { Scene } from "@/editor/blueprints/Scene";
import RhaiEditor from "@/editor/editor/RhaiEditor.vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import CharacterAnimationNodeInspector from "@/editor/editor/CharacterAnimationNodeInspector.vue";

const {
  selected_module_id,
  selected_tileset_path,
  selected_tile_id,
  module_instance_map,
  current_main_instance,
  selected_tile_position,
  selected_tile_layer,
  tile_brush,
  selected_scene_props,
  selected_nav_top_tab,
  active_component,
  inspecting_worlds,
} = storeToRefs(use_editor_store());
const {
  add_open_resource_path,
  set_selected_resource_tab,
  set_current_main_instance,
  set_selected_scene,
  remove_entity_server,
  add_entity_server,
  reset_world,
  set_selected_nav_top_tab,
  set_inspector_component,
  start_inspecting_world,
} = use_editor_store();
const { set_and_render_blueprint_render } = use_game_instances_store();
const rhai_editor = ref<typeof RhaiEditor>();
const selected_script_resource_path = ref<string | null>();
const { game_instance_exists } = use_game_instances_store();

const reset_selected_world = function () {
  if (
    selected_module_id.value &&
    current_main_instance.value &&
    current_main_instance.value.instance_id &&
    current_main_instance.value.world_id
  ) {
    reset_world(
      selected_module_id.value,
      current_main_instance.value.instance_id,
      current_main_instance.value.world_id,
    );
  }
};

const { game_instance_data_map } = storeToRefs(use_game_instances_store());

const selected_character_animation = computed(() => {
  if (
    component_stores.value.character_animation.character_animation_resource_path
  ) {
    return get_or_load_character_animation(
      character_animation_map.value,
      component_stores.value.character_animation
        .character_animation_resource_path,
    );
  }
  return undefined;
});

const { game_map_map, tileset_map, scene_map, character_animation_map } =
  storeToRefs(use_resources_store());
const {
  get_module,
  get_or_load_tileset,
  load_editor_data,
  update_map_server,
  get_or_load_scene,
  get_or_load_map,
  get_or_load_character_animation,
  remove_child_from_scene_on_server,
  add_child_to_scene_on_server,
} = use_resources_store();

const { component_stores } = storeToRefs(use_inspector_store());

// try to load modules until window?.medium?.communication_state?.is_connection_ready is set to true
const load_modules_interval = setInterval(() => {
  if (window?.medium?.communication_state?.is_connection_ready) {
    clearInterval(load_modules_interval);
    setTimeout(() => {
      load_editor_data();
      if (inspecting_worlds.value.main) {
        start_inspecting_world(
          inspecting_worlds.value.main.module_id,
          inspecting_worlds.value.main.instance_id,
          inspecting_worlds.value.main.world_id,
          inspecting_worlds.value.main.map_resource_path,
        );
        // try selecting main instances as long as instance is not loaded
        const interval = setInterval(() => {
          if (
            inspecting_worlds.value.main &&
            !window?.medium?.is_instance_ready(
              inspecting_worlds.value.main.instance_id,
              inspecting_worlds.value.main.world_id,
            )
          ) {
            return;
          }
          if (!selected_scene_props.value.scene_path) {
            return;
          }
          const scene = get_or_load_scene(
            scene_map.value,
            selected_scene_props.value.scene_path,
          );
          if (!scene) {
            return;
          }
          if (!inspecting_worlds.value.main) {
            clearInterval(interval);
            return;
          }
          if (
            game_instance_exists(
              inspecting_worlds.value.main.instance_id,
              inspecting_worlds.value.main.world_id,
            )
          ) {
            clearInterval(interval);
            if (current_main_instance.value.instance_id) {
              select_as_main_instance(
                inspecting_worlds.value.main.module_id,
                inspecting_worlds.value.main.instance_id,
                inspecting_worlds.value.main.world_id,
              );
              console.log(selected_scene_props.value, scene);
              set_and_render_blueprint_render(
                inspecting_worlds.value.main.module_id,
                selected_scene_props.value.scene_path,
                scene,
                selected_scene_props.value.is_pinned,
              );
            }
          }
        }, 100);
      }
    }, 100);
  }
}, 100);

const selected_module = computed(() => get_module(selected_module_id?.value));
const selected_tileset = computed(() =>
  get_or_load_tileset(tileset_map.value, selected_tileset_path.value),
);
const tilesets_of_current_module = computed(() => {
  return selected_module.value.resources
    .filter((r) => r.kind === "Tileset")
    .map((r) => {
      return get_or_load_tileset(tileset_map.value, r.path);
    })
    .filter((r) => r !== undefined) as Tileset[];
});

const scenes_in_module = computed(() => {
  return selected_module.value.resources
    .filter((r) => r.kind === "Scene")
    .map((r) => get_or_load_scene(scene_map.value, r.path));
});

const all_scenes_loaded = computed(() => {
  return scenes_in_module.value.every((s) => s !== undefined);
});

const on_selected_scene_context_menu = (e: MouseEvent) => {
  prevent_browser_default(e);
  if (scenes_in_module.value?.length > 0) {
    if (!all_scenes_loaded.value) {
      setTimeout(() => {
        on_selected_scene_context_menu(e);
      }, 100);
      return;
    }
    ContextMenu.showContextMenu({
      theme: "dark",
      x: e.x,
      y: e.y,
      items: (scenes_in_module.value as Scene[]).map((s) => ({
        label: s.name,
        onClick() {
          set_selected_scene(scene_key(s));
        },
      })),
    });
  }
};

function on_remove_node_from_scene(
  scene_resource: string,
  path: number[],
  node: GameNodeKind,
  is_from_current_instance: boolean,
) {
  if (is_from_current_instance) {
    const entity_id = get_generic_game_node(node).entity_id;
    if (
      selected_module_id.value &&
      current_main_instance.value?.instance_id &&
      current_main_instance.value?.world_id &&
      entity_id
    ) {
      remove_entity_server(
        selected_module_id.value,
        current_main_instance.value.instance_id,
        current_main_instance.value.world_id,
        entity_id,
      );
    } else {
      console.error("Could not attempt to remove entity on server... weird.");
    }
  } else {
    remove_child_from_scene_on_server(scene_resource, path, node);
  }
}

function on_edit_script(script_resource_path: string) {
  selected_script_resource_path.value = script_resource_path;
  setTimeout(() => {
    if (rhai_editor.value) {
      rhai_editor.value.open();
    }
  }, 10);
}

function on_add_node_to_scene(
  scene_resource: string,
  path: number[],
  parent_game_node_id: string,
  parent_entity_id: Entity | null,
  node: GameNodeKind,
  is_from_current_instance: boolean,
) {
  if (is_from_current_instance) {
    if (
      selected_module_id.value &&
      current_main_instance.value?.instance_id &&
      current_main_instance.value?.world_id &&
      typeof parent_entity_id === "number"
    ) {
      add_entity_server(
        selected_module_id.value,
        current_main_instance.value.instance_id,
        current_main_instance.value.world_id,
        parent_entity_id,
        node,
      );
    } else {
      console.error("Could not attempt to add entity on server... weird.");
    }
  } else {
    add_child_to_scene_on_server(
      scene_resource,
      path,
      parent_game_node_id,
      node,
    );
  }
}

function prevent_browser_default(e: MouseEvent) {
  e.preventDefault();
}

function load_map_palette() {
  set_inspector_component("map");
}
const selected_scene = computed(() => {
  if (selected_scene_props.value.scene_path !== null) {
    return get_or_load_scene(
      scene_map.value,
      selected_scene_props.value.scene_path,
    );
  }

  return null;
});
const current_main_instance_scene = computed(() => {
  const { instance_id, world_id } = current_main_instance.value;

  if (
    instance_id &&
    world_id &&
    game_instance_data_map.value[instance_id] &&
    game_instance_data_map.value[instance_id][world_id]
  ) {
    const game_instance_data =
      game_instance_data_map.value[instance_id][world_id];
    if (game_instance_data) {
      return game_instance_data.instance_scene;
    }
  }

  return null;
});
const current_main_map = computed<GameMap | undefined>(() => {
  if (inspecting_worlds.value.main) {
    return get_or_load_map(
      game_map_map.value,
      inspecting_worlds.value.main.map_resource_path,
    );
  }
  return undefined;
});

const scene_for_selected_node = computed(() => {
  if (component_stores.value.game_node.is_instance) {
    return current_main_instance_scene.value;
  }
  return selected_scene.value;
});

const selected_node = computed(() => {
  if (
    component_stores.value.game_node.selection_path &&
    scene_for_selected_node.value?.root_node
  ) {
    return get_node_by_path(scene_for_selected_node.value.root_node, [
      ...component_stores.value.game_node.selection_path,
    ]);
  }
  return undefined;
});

const selected_node_resource_path = computed(() => {
  return component_stores.value.game_node.scene_resource_path;
});

const selected_node_is_instance = computed(() => {
  return component_stores.value.game_node.is_instance === true;
});

const selected_game_node = computed(() => {
  if (selected_node.value) {
    return get_generic_game_node(selected_node.value);
  }
  return undefined;
});
const selected_game_node_path = computed(() => {
  if (
    component_stores.value.game_node.selection_path &&
    current_main_instance_scene.value?.root_node
  ) {
    return component_stores.value.game_node.selection_path;
  }
  return undefined;
});

watch(selected_tile_position, () =>
  on_tile_click(
    selected_tile_layer.value,
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
        layer_parallax: null,
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
    if (current_main_map.value?.main_scene) {
      set_selected_scene(current_main_map.value.main_scene);
    }
  }
}

function open_resource_editor(resource: BlueprintResource) {
  selected_nav_top_tab.value = "resources";
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
  width: 250px;
  height: calc(100vh - 48px);
  background-color: rgb(var(--v-theme-primary));
}

.editor-nav-right > div {
  height: 100%;
}

.editor-nav-top,
.editor-nav-left,
.editor-nav-right {
  pointer-events: all;
}
</style>
