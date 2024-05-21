<template>
  <div>
    <v-label class="form-label">Instance</v-label>
    <v-select
      label="Scene"
      :hide-details="true"
      :items="scene_options"
      :item-title="'file_name'"
      :item-value="'path'"
      :model-value="data"
      @update:model-value="(new_value) => update_path(new_value)"
    ></v-select>
  </div>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { storeToRefs } from "pinia";
import { use_editor_store } from "@/editor/stores/editor";
import { use_resources_store } from "@/editor/stores/resources";

const props = defineProps<{
  data: string;
  is_instance: boolean;
}>();
const { data } = toRefs(props);
const { get_module } = use_resources_store();
const { selected_module_id } = storeToRefs(use_editor_store());
const scene_options = computed(() => {
  const module = get_module(selected_module_id.value);
  if (module) {
    return [
      null,
      ...module.resources.filter((r) => r.kind === "Scene").map((r) => r.path),
    ];
  }
  return [null];
});

const emit = defineEmits<{
  (e: "entityUpdate", data: EntityUpdateKind): void;
}>();

function update_path(name: string) {
  emit("entityUpdate", { InstancePath: name });
}
</script>
<style></style>
