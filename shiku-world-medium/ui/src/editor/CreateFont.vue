<template>
  <v-card class="create-font" title="Create new Font">
    <v-text-field
      label="Name"
      density="comfortable"
      v-model="font.name"
    ></v-text-field>
    <v-text-field
      label="Font Path"
      density="comfortable"
      v-model="font.font_path"
    ></v-text-field>
    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn text="Create Font" @click="save_font"></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts" setup>
import { reactive, toRefs } from "vue";
import { Font } from "@/editor/blueprints/Font";
import { Module } from "@/editor/blueprints/Module";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);

const font = reactive<Font>({
  name: "",
  resource_path: "",
  font_path: "",
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", font: Font): void;
}>();
function save_font() {
  font.resource_path = `modules/${module.value.name}`;
  emit("save", font);
}
</script>

<style>
.create-font {
  display: flex;
  flex-direction: column;
}
</style>
