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
    <v-number-input
      control-variant="stacked"
      :step="0.01"
      :hide-details="true"
      density="compact"
      :label="'Offset'"
      :model-value="data.kinematic_character_controller_props.offset"
      @update:model-value="
        (newValue) => update_kinematic_body_props('offset', newValue)
      "
    ></v-number-input>
    <v-switch
      :hide-details="true"
      density="compact"
      :label="'Slide'"
      :model-value="data.kinematic_character_controller_props.slide"
      @update:model-value="
        (newValue) => update_kinematic_body_props('slide', !!newValue)
      "
    ></v-switch>
    <v-switch
      :hide-details="true"
      density="compact"
      :label="'Snap-to-ground'"
      :model-value="snap_to_ground"
      @update:model-value="toggle_snap_to_ground"
    ></v-switch>
    <v-number-input
      v-if="snap_to_ground"
      control-variant="stacked"
      :step="0.01"
      :hide-details="true"
      density="compact"
      :label="'Snap-to-ground offset'"
      :model-value="data.kinematic_character_controller_props.snap_to_ground"
      @update:model-value="
        (newValue) => update_kinematic_body_props('snap_to_ground', newValue)
      "
    ></v-number-input>
  </div>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { RigidBody } from "@/editor/blueprints/RigidBody";
import { RigidBodyType } from "@/editor/blueprints/RigidBodyType";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { KinematicCharacterControllerProps } from "@/editor/blueprints/KinematicCharacterControllerProps";
const snap_to_ground = computed(() => {
  return !!data.value.kinematic_character_controller_props?.snap_to_ground;
});
function toggle_snap_to_ground(new_value: boolean | null) {
  if (new_value === null) {
    return;
  }
  if (!data.value.kinematic_character_controller_props) {
    return;
  }
  if (!snap_to_ground.value && new_value) {
    update_kinematic_body_props("snap_to_ground", 0.1);
    return;
  }
  if (snap_to_ground.value && !new_value) {
    update_kinematic_body_props("snap_to_ground", null);
    return;
  }
}
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
