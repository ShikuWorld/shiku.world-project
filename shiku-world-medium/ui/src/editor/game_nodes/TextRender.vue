<template>
  <v-text-field
    label="Text"
    :hide-details="true"
    density="compact"
    :model-value="text_render.text"
    @update:model-value="update_text"
  ></v-text-field>
  <v-text-field
    label="Font family"
    :hide-details="true"
    density="compact"
    :model-value="text_render.font_family"
    @update:model-value="update_font_family"
  ></v-text-field>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="Font size"
    :step="1"
    :model-value="text_render.size"
    @update:model-value="update_text_size"
  ></v-number-input>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="Letter spacing"
    :step="1"
    :model-value="text_render.letter_spacing"
    @update:model-value="update_letter_spacing"
  ></v-number-input>
  <v-select
    label="Text align"
    :hide-details="true"
    :items="align_options"
    :model-value="text_render.align"
    @update:model-value="update_align"
  ></v-select>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { TextRender } from "@/editor/blueprints/TextRender";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { TextRenderAlignment } from "@/editor/blueprints/TextRenderAlignment";

const props = defineProps<{ text_render: TextRender }>();
const { text_render } = toRefs(props);

const emit = defineEmits<{
  (e: "updateTextRender", data: TextRender): void;
}>();

const align_options: TextRenderAlignment[] = [
  "Left",
  "Center",
  "Right",
  "Justify",
];

function update_text(text: string) {
  emit("updateTextRender", { ...text_render.value, text });
}

function update_font_family(font_family: string) {
  emit("updateTextRender", { ...text_render.value, font_family });
}

function update_text_size(size: number) {
  emit("updateTextRender", { ...text_render.value, size });
}

function update_align(align: TextRenderAlignment) {
  emit("updateTextRender", { ...text_render.value, align });
}

function update_letter_spacing(letter_spacing: number) {
  emit("updateTextRender", { ...text_render.value, letter_spacing });
}
</script>
<style></style>
