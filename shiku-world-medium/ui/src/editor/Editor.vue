<template>
  <div class="editor-wrapper">
    <div class="editor-nav-top">
      <v-tabs v-model="tab" bg-color="primary">
        <v-tab value="current">Current</v-tab>
        <v-tab value="modules">Modules</v-tab>
      </v-tabs>
    </div>
    <div class="editor-nav-left"></div>
    <div class="editor-main-view">
      <v-window v-model="tab">
        <v-window-item value="Current"></v-window-item>
        <v-window-item value="modules">
          <ModulesEditor class="modules-editor"></ModulesEditor>
        </v-window-item>
      </v-window>
    </div>
    <div class="editor-nav-right">
      {{ selected_module ? selected_module.id : "None" }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import ModulesEditor from "@/editor/editor/ModulesEditor.vue";
import { onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { use_modules_editor_store } from "@/editor/stores/modules_editor";
const tab = ref<number>(0);
const { selected_module } = storeToRefs(use_modules_editor_store());

onMounted(() => {});
</script>

<style lang="scss">
.editor-wrapper {
  display: flex;
  flex-wrap: wrap;
}

.modules-editor {
  height: 100vh;
  pointer-events: all;
}

.editor-main-view {
  display: inline-block;
  flex-grow: 1;
}

.editor-nav-top {
  width: 100%;
  display: block;
}

.editor-nav-left,
.editor-nav-right {
  width: 200px;
  height: 100vh;
  background-color: rgb(var(--v-theme-primary));
}

.editor-nav-top,
.editor-nav-left,
.editor-nav-right {
  pointer-events: all;
}
</style>
