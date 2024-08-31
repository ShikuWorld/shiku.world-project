<template>
  <v-app class="medium-gui">
    <v-main class="medium-main">
      <Editor v-if="editor.editor_open"></Editor>
      <MediumComponent
        v-if="ui.menu_open && ui.current_menu"
        :component_config="ui.current_menu"
        :context="context"
      ></MediumComponent>
    </v-main>
    <medium-toast class="medium-toast"></medium-toast>
  </v-app>
</template>

<style lang="scss">
.medium-gui {
  max-height: 100%;
}

.medium-main {
  max-height: 100%;
}

.medium-toast {
  position: absolute;
  right: 0;
  top: 0;
}

.medium-editor {
  width: 100%;
  height: 100%;
  pointer-events: none;
}
</style>

<script lang="ts" setup>
import MediumComponent from "@/editor/components/MediumComponent.vue";
import { use_ui_store } from "@/editor/stores/ui";
import { DataContext } from "@/editor/ui";
import { use_current_module_store } from "@/editor/stores/current-module";
import MediumToast from "@/editor/toast/MediumToast.vue";
import Editor from "@/editor/editor/Editor.vue";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";
import { computed, Ref } from "vue";
/*import { use_toast_store } from "@/editor/stores/toast";
import { test_menu } from "@/editor/ui/test_menu";*/

const ui = use_ui_store();
const editor = use_editor_store();
const current_module = use_current_module_store();
const { current_menu_context } = storeToRefs(use_ui_store());
const context: Ref<DataContext> = computed(() => {
  console.log(current_menu_context?.value);
  return {
    menu_context: current_menu_context?.value,
    current_module,
    ui,
  };
});

/*
const toast_store = use_toast_store();

toast_store.add_toast("test", "Info", 50000);
window.setInterval(() => {
  toast_store.add_toast("Some crazy things happened!", "Info", 50000);
}, 30000);

current_module.set_data({ someState: 1 });
ui.set_menu(test_menu);
ui.open_menu();
*/
</script>
