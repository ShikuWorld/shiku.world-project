<template>
  <v-select
    label="Body type"
    :hide-details="true"
    :items="body_options"
    :model-value="game_node.data.body"
    @update:model-value="(newValue) => update_body('body', newValue)"
  ></v-select>
  <v-label class="form-label">Position</v-label>
  <v-text-field
    type="number"
    :prepend-icon="mdiAlphaXBox"
    :hide-details="true"
    density="compact"
    :model-value="game_node.data.position[0]"
    @update:model-value="
      (newValue) =>
        update_body('position', [Number(newValue), game_node.data.position[1]])
    "
  ></v-text-field>
  <v-text-field
    :prepend-icon="mdiAlphaYBox"
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="game_node.data.position[1]"
    @update:model-value="
      (newValue) =>
        update_body('position', [game_node.data.position[0], Number(newValue)])
    "
  ></v-text-field>
  <v-label class="form-label">Velocity</v-label>
  <v-text-field
    :prepend-icon="mdiAlphaXBox"
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="game_node.data.velocity[0]"
    @update:model-value="
      (newValue) =>
        update_body('velocity', [Number(newValue), game_node.data.velocity[1]])
    "
  ></v-text-field>
  <v-text-field
    :prepend-icon="mdiAlphaYBox"
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="game_node.data.velocity[1]"
    @update:model-value="
      (newValue) =>
        update_body('velocity', [game_node.data.velocity[0], Number(newValue)])
    "
  ></v-text-field>
  <v-label class="form-label">Rotation</v-label>
  <v-text-field
    type="number"
    density="compact"
    :hide-details="true"
    :model-value="game_node.data.rotation"
    @update:model-value="
      (newValue) => update_body('rotation', Number(newValue))
    "
  ></v-text-field>
</template>
<script lang="ts" setup>
import { onMounted, toRaw, toRefs } from "vue";
import { GameNode } from "@/editor/blueprints/GameNode";
import { RigidBody } from "@/editor/blueprints/RigidBody";
import { RigidBodyType } from "@/editor/blueprints/RigidBodyType";
import { mdiAlphaXBox, mdiAlphaYBox } from "@mdi/js";
const props = defineProps<{ game_node: GameNode<RigidBody> }>();
const { game_node } = toRefs(props);
const body_options: Array<RigidBodyType> = [
  "Dynamic",
  "Fixed",
  "KinematicPositionBased",
  "KinematicVelocityBased",
];
const emit = defineEmits<{
  (e: "dataUpdate", data: RigidBody): void;
}>();

onMounted(() => {
  console.log("huh?");
});

function update_body(key: keyof RigidBody, newValue: unknown) {
  const update = {
    ...{
      body: game_node.value.data.body,
      position: toRaw(game_node.value.data.position),
      velocity: toRaw(game_node.value.data.velocity),
      rotation: game_node.value.data.rotation,
    },
    [key]: newValue,
  };
  emit("dataUpdate", update);
}
</script>
<style>
.form-label {
  margin: 8px 0;
}
</style>
