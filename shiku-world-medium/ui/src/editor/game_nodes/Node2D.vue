<template>
  <div>
    <div>{{ node_2d_type }}</div>
    <v-label class="form-label">Transform</v-label>
    <v-label class="form-label">Position</v-label>
    <v-text-field
      type="number"
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
    ></v-text-field>
    <v-text-field
      :prepend-icon="mdiAlphaYBox"
      type="number"
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
    ></v-text-field>
    <v-label class="form-label">Rotation</v-label>
    <v-text-field
      type="number"
      density="compact"
      :hide-details="true"
      :model-value="game_node.data.transform.rotation"
      @update:model-value="
        (newValue) => update_transform('rotation', Number(newValue))
      "
    ></v-text-field>
  </div>
  <div v-if="node_2d_type !== 'Node2D'">
    <component
      :is="node_2d_component"
      v-bind="{ data }"
      @dataUpdate="data_updated"
      :key="game_node.id"
    ></component>
  </div>
</template>
<script lang="ts" setup>
import { computed, defineAsyncComponent, toRefs } from "vue";
import { Node2D } from "@/editor/blueprints/Node2D";
import { GameNode } from "@/editor/blueprints/GameNode";
import { Node2DKind } from "@/editor/blueprints/Node2DKind";
import { mdiAlphaXBox, mdiAlphaYBox } from "@mdi/js";
import { Transform } from "@/editor/blueprints/Transform";

const props = defineProps<{
  game_node: GameNode<Node2D>;
}>();
const { game_node } = toRefs(props);

const node_2d_type = computed(() => Object.keys(game_node.value.data.kind)[0]);

const data = computed(() => Object.values(game_node.value.data.kind)[0]);

const node_2d_component = computed(() => {
  return defineAsyncComponent(
    () => import(/* @vite-ignore */ `./${node_2d_type.value}.vue`),
  );
});

const emit = defineEmits<{
  (e: "dataUpdate", data: Node2D): void;
}>();

function update_transform(key: keyof Transform, newValue: unknown) {
  const transform = {
    ...game_node.value.data.transform,
    [key]: newValue,
  };
  emit("dataUpdate", { transform, kind: game_node.value.data.kind });
}

function data_updated(kind: Node2DKind) {
  emit("dataUpdate", { transform: game_node.value.data.transform, kind });
}
</script>
<style></style>
