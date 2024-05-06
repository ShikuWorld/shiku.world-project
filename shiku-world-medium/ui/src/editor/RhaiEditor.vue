<script setup lang="ts">
import { ref } from "vue";
import { EditorView, minimalSetup } from "codemirror";
const editor = ref<HTMLElement>();
const is_active = ref<boolean>(false);
function open() {
  is_active.value = true;
  setTimeout(() => {
    if (editor.value) {
      new EditorView({
        doc: 'console.log("hi");',
        extensions: [minimalSetup],
        parent: editor.value,
      });
    }
  }, 100);
}
defineExpose({ open });
</script>

<template>
  <v-dialog max-width="800" v-model="is_active" :scrim="'#ffffff'">
    <v-card title="Some Code">
      <div ref="editor"></div>
      <v-card-actions>
        <v-spacer></v-spacer>

        <v-btn text="Save" @click="is_active = false"></v-btn>
        <v-btn text="Close Dialog" @click="is_active = false"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped></style>
