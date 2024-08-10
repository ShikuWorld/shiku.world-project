<template>
  <div>Collider</div>
  <v-checkbox
    label="Sensor"
    :hide-details="true"
    :model-value="is_sensor"
    @update:model-value="(newValue) => update_is_sensor(newValue ?? false)"
  ></v-checkbox>
  <v-select
    label="Shape"
    :hide-details="true"
    :items="collider_shapes"
    :model-value="collider_shape_kind"
    @update:model-value="(newValue) => update_collider_shape_kind(newValue)"
  ></v-select>
  <v-textarea
    :model-value="collision_data_tmp"
    @update:model-value="(new_value) => (collision_data_tmp = new_value)"
  ></v-textarea>
  <v-btn @click="update_collider_shape_kind_value(collision_data_tmp)"
    >Save</v-btn
  >
</template>
<script lang="ts" setup>
import { Collider } from "@/editor/blueprints/Collider";
import { computed, onMounted, ref, toRefs, watch } from "vue";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { ColliderKind } from "@/editor/blueprints/ColliderKind";
import { KeysOfUnion } from "@/editor/utils";
import { ColliderShape } from "@/editor/blueprints/ColliderShape";
import { match } from "ts-pattern";

type ShapeKind = KeysOfUnion<ColliderShape>;
const props = defineProps<{ data: Collider; is_instance: boolean }>();
const { data } = toRefs(props);
const is_sensor = computed(() => data.value.kind === "Sensor");
const collider_shape_kind = computed(
  () => Object.keys(data.value.shape)[0] as ShapeKind,
);
const collision_data_tmp = ref<string | null | undefined>();
const collider_shapes: Array<ShapeKind> = [
  "Ball",
  "CapsuleX",
  "CapsuleY",
  "Cuboid",
];
const emit = defineEmits<{
  (e: "entityUpdate", data: EntityUpdateKind): void;
}>();

const update_is_sensor = (is_sensor: boolean | null) => {
  emit("entityUpdate", {
    Collider: {
      kind: (is_sensor ? "Sensor" : "Solid") as ColliderKind,
      shape: data.value.shape,
    },
  });
};

const update_collider_shape_kind = (shape_kind: ShapeKind | null) => {
  if (shape_kind === null) {
    return;
  }
  emit("entityUpdate", {
    Collider: {
      kind: data.value.kind,
      shape: create_collider_shape(shape_kind),
    },
  });
};

const update_collider_shape_kind_value = (
  json_value: string | null | undefined,
) => {
  try {
    const value = JSON.parse(json_value ?? "");
    emit("entityUpdate", {
      Collider: {
        kind: data.value.kind,
        shape: { [collider_shape_kind.value]: value } as ColliderShape,
      },
    });
  } catch (e) {
    console.error(e);
  }
};

const collision_data = computed(() => {
  try {
    if (data.value && data.value.shape) {
      return JSON.stringify(Object.values(data.value.shape)[0]);
    }
  } catch (e) {
    return null;
  }
});

watch(data, () => {
  collision_data_tmp.value = collision_data.value;
});

onMounted(() => {
  collision_data_tmp.value = collision_data.value;
});

const create_collider_shape = (
  shape_kind: KeysOfUnion<ColliderShape>,
): ColliderShape => {
  return match(shape_kind)
    .with("Ball", (): ColliderShape => ({ Ball: 1.0 }))
    .with("CapsuleX", (): ColliderShape => ({ CapsuleX: [1.0, 1.0] }))
    .with("CapsuleY", (): ColliderShape => ({ CapsuleY: [1.0, 1.0] }))
    .with("Cuboid", (): ColliderShape => ({ Cuboid: [1.0, 1.0] }))
    .exhaustive();
};
</script>
<style></style>
