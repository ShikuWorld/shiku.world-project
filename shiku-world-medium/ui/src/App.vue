<template>
  <v-app class="medium-gui">
    <v-main class="medium-main" v-if="ui.menu_open && ui.current_menu">
      <MediumComponent
        :component_config="ui.current_menu"
        :context="context"
      ></MediumComponent>
    </v-main>
    <div class="rete" ref="rete"></div>
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

.rete {
  width: 100vw;
  height: 100vh;
  pointer-events: all;
}
</style>

<script lang="ts" setup>
import MediumComponent from "@/editor/components/MediumComponent.vue";
import { use_ui_store } from "@/editor/stores/ui";
import { DataContext } from "@/editor/ui";
import { use_current_module_store } from "@/editor/stores/current-module";
import MediumToast from "@/editor/toast/MediumToast.vue";
import { onMounted, onUnmounted, ref } from "vue";
import { ModuleEditor } from "@/editor/editor";
/*import { use_toast_store } from "@/editor/stores/toast";
import { test_menu } from "@/editor/ui/test_menu";*/

const rete = ref(null);
const ui = use_ui_store();
const current_module = use_current_module_store();
const module_editor = new ModuleEditor();

const context: DataContext = {
  current_module,
  ui,
};

onMounted(async () => {
  await module_editor.initRender(rete.value as unknown as HTMLElement);
  await module_editor.addModuleNode({
    exit_points: [{ name: "ToGame1", condition_script: "" }],
    insert_points: [{ name: "FromLogin", condition_script: "" }],
    name: "Lobby",
    maps: [],
    max_guests: 0,
    min_guests: 0,
    resources: [],
  });
  await module_editor.addModuleNode({
    exit_points: [{ name: "Exit1", condition_script: "" }],
    insert_points: [
      { name: "Entry1", condition_script: "" },
      { name: "Entry2", condition_script: "" },
    ],
    name: "Dummy",
    maps: [],
    max_guests: 0,
    min_guests: 0,
    resources: [],
  });
  await module_editor.layout();
});

onUnmounted(() => {
  module_editor.destroy();
});

/*
const toast_store = use_toast_store();

toast_store.add_toast("test", "Info", 50000);
setInterval(() => {
  toast_store.add_toast("Some crazy things happened!", "Info", 50000);
}, 30000);

current_module.set_data({ someState: 1 });
ui.set_menu(test_menu);
ui.open_menu();
*/
</script>
