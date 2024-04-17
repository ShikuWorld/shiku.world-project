<template>
  <v-label class="form-label">Rigid Body</v-label>
  <v-select
    label="Body type"
    :hide-details="true"
    :items="body_options"
    :model-value="data.body"
    @update:model-value="(newValue) => update_body_type(newValue)"
  ></v-select>
  <v-label class="form-label">Velocity</v-label>
  <v-text-field
    :prepend-icon="mdiAlphaXBox"
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="data.velocity[0]"
    @update:model-value="
      (newValue) =>
        update_body('velocity', [Number(newValue), data.velocity[1]])
    "
  ></v-text-field>
  <v-text-field
    :prepend-icon="mdiAlphaYBox"
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="data.velocity[1]"
    @update:model-value="
      (newValue) =>
        update_body('velocity', [data.velocity[0], Number(newValue)])
    "
  ></v-text-field>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { RigidBody } from "@/editor/blueprints/RigidBody";
import { RigidBodyType } from "@/editor/blueprints/RigidBodyType";
import { mdiAlphaXBox, mdiAlphaYBox } from "@mdi/js";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";

const props = defineProps<{ data: RigidBody }>();
const { data } = toRefs(props);
const body_options: Array<RigidBodyType> = [
  "Dynamic",
  "Fixed",
  "KinematicPositionBased",
  "KinematicVelocityBased",
];
const emit = defineEmits<{
  (e: "entityUpdate", data: EntityUpdateKind): void;
}>();

function update_body(_key: keyof RigidBody, _newValue: unknown) {
  console.log("This does nothing atm :)");
}

function update_body_type(rigid_body_type: RigidBodyType) {
  emit("entityUpdate", { RigidBodyType: rigid_body_type });
}
</script>
<style>
.form-label {
  margin: 8px 0;
}
</style>
