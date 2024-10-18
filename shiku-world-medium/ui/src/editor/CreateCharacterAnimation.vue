<template>
  <v-card
    class="create-character-animation"
    title="Create new Character Animation"
  >
    <v-text-field
      label="Name"
      v-model="character_animation.name"
    ></v-text-field>
    <v-text-field
      label="Resource Path"
      density="comfortable"
      v-model="character_animation.resource_path"
    ></v-text-field>
    <v-select
      label="Tileset Resource"
      v-model="character_animation.tileset_resource"
      :items="tileset_options"
    ></v-select>
    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn
        text="Create Character Animation"
        @click="save_character_animation"
      ></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts" setup>
import { CharacterAnimation } from "@/client/communication/api/blueprints/CharacterAnimation";
import { reactive, toRefs, computed } from "vue";
import { Module } from "@/editor/blueprints/Module";

const props = defineProps<{ module: Module }>();
const { module } = toRefs(props);
const tileset_options = computed(() =>
  module.value.resources.filter((r) => r.kind === "Tileset").map((r) => r.path),
);
const character_animation = reactive<CharacterAnimation>({
  id: "default_id",
  name: "",
  resource_path: "",
  tileset_resource: "",
  start_direction: "Right",
  start_state: 0,
  states: { 0: { name: "Idle", frames: [], loop_animation: false } },
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", character_animation: CharacterAnimation): void;
}>();
function save_character_animation() {
  emit("save", character_animation);
}
</script>

<style>
.create-character-animation {
  display: flex;
  flex-direction: column;
}
</style>
