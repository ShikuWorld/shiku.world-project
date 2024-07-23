<template>
  <v-number-input
    :reverse="false"
    :label="label"
    controlVariant="stacked"
    density="compact"
    :hide-details="true"
    :step="1"
    :model-value="modelValue"
    @update:focused="() => emit('focus')"
    @update:model-value="(new_value) => emit('update:modelValue', new_value)"
  >
    <template v-slot:append>
      <TilePreview :tileset="tileset" :tile_id="modelValue" />
    </template>
  </v-number-input>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import TilePreview from "@/editor/editor/TilePreview.vue";
import { VNumberInput } from "vuetify/labs/VNumberInput";

interface Props {
  tileset: Tileset;
  label: string;
  modelValue: number;
}

const props = defineProps<Props>();
const { modelValue, tileset, label } = toRefs(props);
const emit = defineEmits(["update:modelValue", "focus", "blur"]);
</script>
