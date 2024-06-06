<template>
  <div>
    <v-text-field
      :hide-details="true"
      density="compact"
      :model-value="game_node.name"
      @update:model-value="(new_value) => update_name(new_value)"
    ></v-text-field>
    <v-select
      label="Script"
      :hide-details="true"
      :items="scripts"
      :model-value="game_node.script"
      @update:model-value="(newValue) => update_script(newValue)"
    ></v-select>
    <v-label class="form-label">Transform</v-label>
    <v-label class="form-label">Position</v-label>
    <v-number-input
      control-variant="stacked"
      :step="0.1"
      :prepend-icon="mdiAlphaXBox"
      :hide-details="true"
      density="compact"
      :model-value="game_node.data.transform.position[0]"
      @update:model-value="
        (newValue) =>
          update_transform('position', [
            Number(newValue),
            game_node.data.transform.position[1],
          ])
      "
    ></v-number-input>
    <v-number-input
      :prepend-icon="mdiAlphaYBox"
      control-variant="stacked"
      :step="0.1"
      :hide-details="true"
      density="compact"
      :model-value="game_node.data.transform.position[1]"
      @update:model-value="
        (newValue) =>
          update_transform('position', [
            game_node.data.transform.position[0],
            Number(newValue),
          ])
      "
    ></v-number-input>
    <v-label class="form-label">Rotation</v-label>
    <v-number-input
      control-variant="stacked"
      :step="0.01"
      density="compact"
      :hide-details="true"
      :model-value="game_node.data.transform.rotation"
      @update:model-value="
        (newValue) => update_transform('rotation', Number(newValue))
      "
    ></v-number-input>
  </div>
  <div v-if="node_2d_type !== 'Node2D'">
    <component
      :is="node_2d_component"
      v-bind="{ data, is_instance }"
      @entityUpdate="entity_updated"
      :key="game_node.id"
    ></component>
  </div>
</template>
<script lang="ts" setup>
import { computed, defineAsyncComponent, toRefs } from "vue";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { Node2D } from "@/editor/blueprints/Node2D";
import { GameNode } from "@/editor/blueprints/GameNode";
import { mdiAlphaXBox, mdiAlphaYBox } from "@mdi/js";
import { Transform } from "@/editor/blueprints/Transform";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { storeToRefs } from "pinia";
import { use_editor_store } from "@/editor/stores/editor";
import { use_resources_store } from "@/editor/stores/resources";

const props = defineProps<{
  game_node: GameNode<Node2D>;
  is_instance: boolean;
}>();
const { game_node, is_instance } = toRefs(props);
const { get_module } = use_resources_store();
const { selected_module_id } = storeToRefs(use_editor_store());
const scripts = computed(() => {
  const module = get_module(selected_module_id.value);
  if (module) {
    return [
      null,
      ...module.resources.filter((r) => r.kind === "Script").map((r) => r.path),
    ];
  }
  return [null];
});

const node_2d_type = computed(() => Object.keys(game_node.value.data.kind)[0]);

const data = computed(() => Object.values(game_node.value.data.kind)[0]);

const node_2d_component = computed(() => {
  return defineAsyncComponent(
    () => import(/* @vite-ignore */ `./${node_2d_type.value}.vue`),
  );
});

const emit = defineEmits<{
  (e: "entityUpdate", data: EntityUpdateKind): void;
}>();

function update_name(name: string) {
  emit("entityUpdate", { Name: name });
}

function update_script(script_path: string | null) {
  emit("entityUpdate", { ScriptPath: script_path });
}

function update_transform(key: keyof Transform, newValue: unknown) {
  const transform = {
    ...game_node.value.data.transform,
    [key]: newValue,
  };
  emit("entityUpdate", { Transform: transform });
}

function entity_updated(entity_update: EntityUpdateKind) {
  emit("entityUpdate", entity_update);
}
</script>
<style></style>
