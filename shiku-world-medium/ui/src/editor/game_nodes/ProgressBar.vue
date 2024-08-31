<template>
  <v-select
    :hide-details="true"
    :label="'Tileset'"
    density="compact"
    :items="tileset_options"
    :item-title="'file_name'"
    :item-value="'path'"
    :model-value="progress_bar.tileset"
    @update:model-value="
      (new_value) => update_progress_bar('tileset', new_value)
    "
  ></v-select>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="bg Gid"
    :step="1"
    :min="0"
    :model-value="progress_bar.background"
    @update:model-value="
      (new_value) => update_progress_bar('background', new_value)
    "
  ></v-number-input>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="fill Gid"
    :step="1"
    :min="0"
    :model-value="progress_bar.fill"
    @update:model-value="(new_value) => update_progress_bar('fill', new_value)"
  ></v-number-input>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="progress"
    :step="0.1"
    :min="0"
    :max="100"
    :model-value="progress_bar.progress"
    @update:model-value="
      (new_value) => update_progress_bar('progress', new_value)
    "
  ></v-number-input>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="width"
    :step="1"
    :min="0"
    :model-value="progress_bar.width"
    @update:model-value="(new_value) => update_progress_bar('width', new_value)"
  ></v-number-input>
  <v-number-input
    control-variant="stacked"
    density="compact"
    hide-details="auto"
    label="height"
    :step="1"
    :min="0"
    :model-value="progress_bar.height"
    @update:model-value="
      (new_value) => update_progress_bar('height', new_value)
    "
  ></v-number-input>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { ProgressBar } from "@/editor/blueprints/ProgressBar";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import { ProgressBarUpdate } from "@/editor/blueprints/ProgressBarUpdate";
import { VNumberInput } from "vuetify/labs/VNumberInput";

const props = defineProps<{
  progress_bar: ProgressBar;
  tileset_options: BlueprintResource[];
}>();
const { progress_bar, tileset_options } = toRefs(props);

const emit = defineEmits<{
  (e: "progressBarUpdate", data: ProgressBarUpdate): void;
}>();

function update_progress_bar(key: keyof ProgressBar, value: unknown) {
  emit("progressBarUpdate", {
    progress: null,
    fill: null,
    fill_paddings: null,
    background: null,
    tileset: null,
    width: null,
    height: null,
    [key]: value,
  });
}
</script>
<style></style>
