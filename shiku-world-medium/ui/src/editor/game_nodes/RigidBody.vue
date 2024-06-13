<template>
  <v-label class="form-label">Rigid Body</v-label>
  <v-select
    label="Body type"
    :hide-details="true"
    :items="body_options"
    :model-value="data.body"
    @update:model-value="(newValue) => update_body_type(newValue)"
  ></v-select>
  <div v-if="data.kinematic_character_controller_props">
    <v-label class="form-label">Kinematic props</v-label>
    <v-number-input
      control-variant="stacked"
      :step="0.1"
      :hide-details="true"
      density="compact"
      :label="'Normal Nudge Factor'"
      :model-value="
        data.kinematic_character_controller_props.normal_nudge_factor
      "
      @update:model-value="
        (newValue) =>
          update_kinematic_body_props('normal_nudge_factor', newValue)
      "
    ></v-number-input>
  </div>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { RigidBody } from "@/editor/blueprints/RigidBody";
import { RigidBodyType } from "@/editor/blueprints/RigidBodyType";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { KinematicCharacterControllerProps } from "@/editor/blueprints/KinematicCharacterControllerProps";

const props = defineProps<{ data: RigidBody; is_instance: boolean }>();
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

function update_kinematic_body_props<
  T extends keyof KinematicCharacterControllerProps,
>(key: T, newValue: KinematicCharacterControllerProps[T]) {
  if (!data.value.kinematic_character_controller_props) {
    return;
  }
  emit("entityUpdate", {
    KinematicCharacterControllerProps: {
      ...data.value.kinematic_character_controller_props,
      [key]: newValue,
    },
  });
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
