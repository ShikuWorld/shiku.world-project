<template>
  <v-card class="create-tileset" title="New Script">
    <v-text-field label="Script name" v-model="script.name"></v-text-field>
    <v-text-field
      label="Resource path"
      v-model="script.resource_path"
    ></v-text-field>
    <v-card-actions>
      <v-spacer></v-spacer>

      <v-btn text="Create Script" @click="save_script"></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>
<script lang="ts" setup>
import { reactive } from "vue";
import { Script } from "@/editor/blueprints/Script";
import { v4 as uuidv4 } from "uuid";

const script = reactive<Script>({
  name: "",
  id: uuidv4(),
  content: "",
  resource_path: "",
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", script: Script): void;
}>();
function save_script() {
  emit("save", script);
}
</script>
<style>
.create-tileset {
  display: flex;
  flex-direction: column;
}
</style>
