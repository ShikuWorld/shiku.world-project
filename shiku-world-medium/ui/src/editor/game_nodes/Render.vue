<template>
  <v-label class="form-label">Render</v-label>
  <v-select
    label="Render type"
    :hide-details="true"
    :items="render_options"
    :model-value="render_kind"
    @update:model-value="(new_value) => update_render_type(new_value)"
  ></v-select>
  <v-label class="form-label">GID</v-label>
  <v-text-field
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="gid"
    @update:model-value="(new_value) => update_gid(new_value)"
  ></v-text-field>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { RenderKind } from "@/editor/blueprints/RenderKind";
import { KeysOfUnion } from "@/editor/utils";
import { Render } from "@/editor/blueprints/Render";
import { match, P } from "ts-pattern";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";

const props = defineProps<{ data: Render }>();
const { data } = toRefs(props);
const render_options: Array<KeysOfUnion<RenderKind>> = [
  "AnimatedSprite",
  "Sprite",
];
const emit = defineEmits<{
  (e: "entityUpdate", data: EntityUpdateKind): void;
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

function update_render_type(kind: KeysOfUnion<RenderKind>) {
  console.log("TODO", kind);
}

function update_gid(gid: string) {
  emit("entityUpdate", { UpdateGid: Number(gid) });
}
</script>
<style></style>
