<template>
  <v-select
    label="Render type"
    :hide-details="true"
    :items="render_options"
    :model-value="render_kind"
    @update:model-value="(newValue) => update_body('kind', { [newValue]: gid })"
  ></v-select>
  <v-label class="form-label">GID</v-label>
  <v-text-field
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="gid"
    @update:model-value="
      (newValue) => update_body('kind', { [render_kind]: Number(newValue) })
    "
  ></v-text-field>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { RenderKind } from "@/editor/blueprints/RenderKind";
import { KeysOfUnion } from "@/editor/utils";
import { Render } from "@/editor/blueprints/Render";
import { match, P } from "ts-pattern";
import { Node2DKind } from "@/editor/blueprints/Node2DKind";

const props = defineProps<{ data: Render }>();
const { data } = toRefs(props);
const render_options: Array<KeysOfUnion<RenderKind>> = [
  "AnimatedSprite",
  "Sprite",
];
const emit = defineEmits<{
  (e: "dataUpdate", data: Node2DKind): void;
}>();

const render_kind = computed(
  () => Object.keys(data.value.kind)[0] as KeysOfUnion<RenderKind>,
);

const gid = computed(() => {
  return match(data.value.kind)
    .with({ Sprite: P.select() }, (s) => s)
    .with({ AnimatedSprite: P.select() }, (s) => s)
    .exhaustive();
});

function update_body(key: keyof Render, newValue: unknown) {
  const update = {
    ...{
      offset: data.value.offset,
      layer: data.value.layer,
      kind: data.value.kind,
    },
    [key]: newValue,
  };
  emit("dataUpdate", { Render: update });
}
</script>
<style></style>
