<template>
  <div class="entities-list">
    <v-btn
      class="scene-editor-entities-list-plus"
      :icon="mdiPlus"
      :id="menu_id"
      density="comfortable"
      color="primary"
      size="small"
    >
    </v-btn>
    <v-menu :activator="menu_id_hash">
      <v-list>
        <v-list-item v-for="node_type in node_type_options">
          <v-list-item-title
            v-if="
              selected_node.selection_path &&
              selected_node.selected_game_node_id
            "
            @click="
              add_node_type(
                selected_node.selection_path,
                selected_node.selected_game_node_id,
                node_type.value,
              )
            "
            >{{ node_type.label }}
          </v-list-item-title>
        </v-list-item>
      </v-list>
    </v-menu>
    <SceneNodeList
      :scene_resource_path="scene_key(scene)"
      :node="scene.root_node"
      :path="[]"
      :node_is_instance="is_node_instance"
      :scene_is_instance="is_scene_instance"
      @remove_node="on_remove_node"
      @edit_script="on_edit_script"
    ></SceneNodeList>
  </div>
  <v-dialog max-width="800" v-model="is_instance_modal_open" :scrim="'#ffffff'">
    <v-card title="New Instance">
      <v-label class="form-label">Name</v-label>
      <v-text-field
        :model-value="instance_name"
        @update:model-value="(new_value) => (instance_name = new_value)"
      ></v-text-field>
      <v-label class="form-label">Blueprint</v-label>
      <v-select
        label="Scene"
        :hide-details="true"
        :items="scene_options"
        :item-title="'file_name'"
        :item-value="'path'"
        :model-value="instance_path"
        @update:model-value="
          (new_value) => {
            instance_path = new_value;
            const resource = scene_options.find((s) =>
              s && s.path === new_value ? s : null,
            );
            if (resource) {
              instance_name = resource.file_name.split('.')[0];
            }
          }
        "
      ></v-select>
      <v-card-actions>
        <v-spacer></v-spacer>

        <v-btn text="Save" @click="on_instance_modal_save"></v-btn>
        <v-btn
          text="Close Dialog"
          @click="is_instance_modal_open = false"
        ></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style>
.entities-list {
  padding: 8px;
}
.scene-editor-entities-list-plus {
  position: absolute;
  bottom: 8px;
  right: 8px;
}
</style>

<script lang="ts" setup>
import { computed, ref, toRefs } from "vue";
import type { Scene } from "@/editor/blueprints/Scene";
import SceneNodeList from "@/editor/editor/SceneNodeList.vue";
import { mdiPlus } from "@mdi/js";
import { storeToRefs } from "pinia";
import { use_inspector_store } from "@/editor/stores/inspector";
import {
  create_game_node,
  GameNodeTypeKeys,
  scene_key,
  use_resources_store,
} from "@/editor/stores/resources";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { Entity } from "@/editor/blueprints/Entity";
import { v4 as uuidv4 } from "uuid";
import { use_editor_store } from "@/editor/stores/editor";

const { get_module } = use_resources_store();
const { selected_module_id } = storeToRefs(use_editor_store());
const scene_options = computed(() => {
  const module = get_module(selected_module_id.value);
  if (module) {
    return [null, ...module.resources.filter((r) => r.kind === "Scene")];
  }
  return [null];
});

const node_type_options: { value: GameNodeTypeKeys; label: string }[] = [
  { value: "Node2D-Node2D", label: "Node 2D" },
  { value: "Node2D-Instance", label: "Node 2D Instance" },
  { value: "Node2D-RigidBody", label: "Node 2D RigidBody" },
  { value: "Node2D-Render", label: "Node 2D Render" },
  { value: "Node2D-Collider", label: "Node 2D Collider" },
];
const props = defineProps<{
  scene: Scene;
  is_scene_instance: boolean;
  menu_id: string;
}>();
const { scene, is_scene_instance, menu_id } = toRefs(props);
const menu_id_hash = computed(() => `#${menu_id.value}`);

const { component_stores } = storeToRefs(use_inspector_store());
const selected_node = computed(() => component_stores.value.game_node);
const is_node_instance = computed(
  () => component_stores.value.game_node.is_instance === true,
);
const is_instance_modal_open = ref<boolean>(false);
const instance_name = ref<string>("");
const instance_path = ref<string | null>(null);

const emit = defineEmits<{
  (
    e: "remove_node",
    scene_resource: string,
    path: number[],
    node: GameNodeKind,
    is_from_current_instance: boolean,
  ): void;
  (e: "edit_script", script_resource_path: string): void;
  (
    e: "add_node",
    scene_resource: string,
    path: number[],
    parent_game_node_id: string,
    parent_node_entity_id: Entity | null,
    node: GameNodeKind,
    is_from_current_instance: boolean,
  ): void;
}>();
function on_remove_node(
  scene_resource: string,
  path: number[],
  node: GameNodeKind,
  is_from_current_instance: boolean,
) {
  emit("remove_node", scene_resource, path, node, is_from_current_instance);
}
function on_edit_script(script_resource_path: string) {
  emit("edit_script", script_resource_path);
}

function on_instance_modal_save() {
  is_instance_modal_open.value = false;
  if (
    !selected_node.value.selection_path ||
    !selected_node.value.selected_game_node_id ||
    !instance_path.value
  ) {
    return;
  }
  emit(
    "add_node",
    scene_key(scene.value),
    selected_node.value.selection_path,
    selected_node.value.selected_game_node_id,
    selected_node.value.selected_entity_id !== undefined
      ? selected_node.value.selected_entity_id
      : null,
    {
      Node2D: {
        name: instance_name.value,
        id: uuidv4(),
        entity_id: null,
        instance_resource_path: null,
        tags: [],
        data: {
          kind: { Instance: instance_path.value },
          transform: {
            position: [0, 0],
            rotation: 0,
            scale: [1, 1],
            velocity: [1, 1],
          },
        },
        script: null,
        children: [],
      },
    },
    is_scene_instance.value,
  );
}

function add_node_type(
  path: number[],
  selected_game_node_id: string,
  node_type: GameNodeTypeKeys,
) {
  if (!path) {
    console.error("Tried to add node to undefined node.");
    return;
  }

  if (node_type === "Node2D-Instance") {
    instance_path.value = null;
    instance_name.value = "";
    is_instance_modal_open.value = true;
    return;
  }

  let game_node = create_game_node(node_type);
  if (!game_node) {
    console.error("Could not create game node to add to scene on server!");
    return;
  }
  emit(
    "add_node",
    scene_key(scene.value),
    path,
    selected_game_node_id,
    selected_node.value.selected_entity_id !== undefined
      ? selected_node.value.selected_entity_id
      : null,
    game_node,
    is_scene_instance.value,
  );
}
</script>
