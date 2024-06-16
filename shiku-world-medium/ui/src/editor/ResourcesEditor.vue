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
            @click="close_resource(resource_key(resource))"
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
            <v-list-item v-if="selected_module">
              <v-list-item-title @click="create_tileset_dialog = true"
                >Tileset</v-list-item-title
              >
            </v-list-item>
            <v-list-item v-if="selected_module">
              <v-list-item-title @click="create_map_dialog = true"
                >Map</v-list-item-title
              >
            </v-list-item>
            <v-list-item v-if="selected_module">
              <v-list-item-title @click="create_scene_dialog = true"
                >Scene</v-list-item-title
              >
            </v-list-item>
            <v-list-item v-if="selected_module">
              <v-list-item-title @click="create_script_dialog = true"
                >Script</v-list-item-title
              >
            </v-list-item>
            <v-list-item v-if="selected_module">
              <v-list-item-title
                @click="create_character_animation_dialog = true"
                >Character Animation</v-list-item-title
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
        <v-dialog v-model="create_map_dialog" width="800">
          <CreateMap
            @close="create_map_dialog = false"
            @save="save_map"
            :module="selected_module"
          ></CreateMap>
        </v-dialog>
        <v-dialog v-model="create_scene_dialog" width="800">
          <CreateScene
            @close="create_scene_dialog = false"
            @save="save_scene"
          ></CreateScene>
        </v-dialog>
        <v-dialog v-model="create_script_dialog" width="800">
          <CreateScript
            @close="create_script_dialog = false"
            @save="save_script"
          ></CreateScript>
        </v-dialog>
        <v-dialog v-model="create_character_animation_dialog" width="800">
          <CreateCharacterAnimation
            :module="selected_module"
            @close="create_character_animation_dialog = false"
            @save="save_character_animation"
          ></CreateCharacterAnimation>
        </v-dialog>
      </v-window-item>
      <v-window-item
        v-for="resource in open_resources"
        :key="resource.path"
        :value="resource.path"
        ><TilesetEditor
          v-if="
            resource.kind === 'Tileset' && tileset_map[resource_key(resource)]
          "
          :tileset="tileset_map[resource_key(resource)]"
          @tile_selected="on_tile_selected"
        ></TilesetEditor
      ></v-window-item>
    </v-window>
  </div>
</template>
<script lang="ts" setup>
import { mdiPlus, mdiTrashCan } from "@mdi/js";
import CreateTileset from "@/editor/editor/CreateTileset.vue";
import { computed, onMounted, ref, watch } from "vue";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { GameMap } from "@/client/communication/api/blueprints/GameMap";
import { resource_key, use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";
import TilesetEditor from "@/editor/editor/TilesetEditor.vue";
import { match } from "ts-pattern";
import CreateMap from "@/editor/editor/CreateMap.vue";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import CreateScene from "@/editor/editor/CreateScene.vue";
import { Scene } from "@/editor/blueprints/Scene";
import { use_resources_store } from "@/editor/stores/resources";
import { Script } from "@/editor/blueprints/Script";
import CreateScript from "@/editor/editor/CreateScript.vue";
import { CharacterAnimation } from "@/editor/blueprints/CharacterAnimation";
import CreateCharacterAnimation from "@/editor/editor/CreateCharacterAnimation.vue";

const create_tileset_dialog = ref(false);
const create_map_dialog = ref(false);
const create_scene_dialog = ref(false);
const create_script_dialog = ref(false);
const create_character_animation_dialog = ref(false);
const { close_resource, set_selected_tile, set_inspector_component } =
  use_editor_store();
const { open_resource_paths, selected_resource_tab, selected_module_id } =
  storeToRefs(use_editor_store());

const {
  create_tileset_server,
  create_map_server,
  create_scene_server,
  get_module,
  get_resource_server,
  create_script_server,
  create_character_animation_server,
} = use_resources_store();
const { modules, tileset_map } = storeToRefs(use_resources_store());
const available_resources = computed(
  () =>
    new Map(
      Object.values(modules.value).flatMap((m) =>
        m.resources.map((resource) => [resource_key(resource), resource]),
      ),
    ),
);
const selected_module = computed(() => get_module(selected_module_id.value));
const open_resources = computed(
  () =>
    open_resource_paths.value.map((path) =>
      available_resources.value.get(path),
    ) as BlueprintResource[],
);

watch(open_resources, () => {
  ensure_resources_are_loaded();
});

onMounted(() => {
  ensure_resources_are_loaded();
});

const on_tile_selected = (g_id: number, tileset_key: string) => {
  set_selected_tile(tileset_key, g_id);
  set_inspector_component("tile");
};

function ensure_resources_are_loaded() {
  for (const resource of open_resources.value) {
    match(resource)
      .with({ kind: "Tileset" }, (r) => {
        if (!tileset_map.value[resource_key(r)]) {
          get_resource_server(r.path);
        }
      })
      .with({ kind: "Map" }, (r) => {
        console.log("hm Map?", r);
      })
      .with({ kind: "Scene" }, (r) => {
        console.log("hm Scene?", r);
      })
      .with({ kind: "Script" }, (r) => {
        console.log("hm Script?", r);
      })
      .with({ kind: "CharacterAnimation" }, (r) => {
        console.log("hm CharacterAnimation?", r);
      })
      .with({ kind: "Unknown" }, () => {})
      .exhaustive();
  }
}

function save_tileset(tileset: Tileset) {
  create_tileset_dialog.value = false;
  create_tileset_server(selected_module_id.value, tileset);
}

function save_map(game_map: GameMap) {
  create_map_dialog.value = false;
  create_map_server(game_map);
}

function save_scene(scene: Scene) {
  create_scene_dialog.value = false;
  create_scene_server(selected_module_id.value, scene);
}

function save_script(script: Script) {
  create_script_dialog.value = false;
  create_script_server(selected_module_id.value, script);
}

function save_character_animation(character_animation: CharacterAnimation) {
  create_character_animation_dialog.value = false;
  create_character_animation_server(
    selected_module_id.value,
    character_animation,
  );
}
</script>
<style></style>
