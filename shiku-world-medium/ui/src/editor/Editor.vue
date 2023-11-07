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
          <ModulesGraph class="modules-editor"></ModulesGraph>
        </v-window-item>
      </v-window>
    </div>
    <div class="editor-nav-right">
      <ModulesEditor
        v-if="selected_module"
        :module="selected_module"
      ></ModulesEditor>
    </div>
  </div>
</template>

<script lang="ts" setup>
import ModulesGraph from "@/editor/editor/ModulesGraph.vue";
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import ModulesEditor from "@/editor/editor/ModulesEditor.vue";
import { use_editor_store } from "@/editor/stores/editor";
const tab = ref<number>(0);
const { selected_module_id } = storeToRefs(use_editor_store());
const { get_module } = use_editor_store();

const selected_module = computed(() => get_module(selected_module_id?.value));
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
