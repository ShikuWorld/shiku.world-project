<template>
  <v-text-field
    :hide-details="true"
    density="compact"
    label="Brush name"
    :model-value="brush_name"
    @update:model-value="(new_value) => update_brush_name(new_value)"
  ></v-text-field>
  <GidInput
    v-for="[kernel_name, gid] in brush_kernels"
    :tileset="tileset"
    :model-value="gid"
    :label="kernel_name"
    @focus="selected_kernel_name = kernel_name"
    @update:model-value="(new_gid) => update_brush_kernel(kernel_name, new_gid)"
  ></GidInput>
</template>
<script lang="ts" setup>
import { computed, ref, toRefs, watch } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { StandardKernelThree } from "@/editor/blueprints/StandardKernelThree";
import GidInput from "@/editor/editor/GidInput.vue";
import { storeToRefs } from "pinia";
import { use_editor_store } from "@/editor/stores/editor";

interface Props {
  tileset: Tileset;
  brush_name: string;
  brush: StandardKernelThree;
}
const selected_kernel_name = ref<string | null>(null);
const props = defineProps<Props>();
const { tileset, brush_name, brush } = toRefs(props);
const { selected_tile_id } = storeToRefs(use_editor_store());

watch(selected_tile_id, (new_value) => {
  if (selected_kernel_name.value !== null) {
    update_brush_kernel(selected_kernel_name.value, new_value);
  }
});

const emit = defineEmits<{
  (e: "brush_kernel_update", brush: StandardKernelThree): void;
  (e: "brush_name_update", brush_name: string): void;
}>();

const update_brush_name = (new_value: string) => {
  emit("brush_name_update", new_value);
};

const update_brush_kernel = (kernel_name: string, new_gid: number) => {
  emit("brush_kernel_update", { ...brush.value, [kernel_name]: new_gid });
};

const brush_kernels = computed(() => {
  return Object.entries(brush.value) as [string, number][];
});
</script>
