<template>
  <div>
    <ComponentEditor
      class="component-editor"
      ref="componentEditor"
    ></ComponentEditor>
  </div>
</template>

<script lang="ts" setup>
import ComponentEditor from "@/editor/editor/ComponentEditor.vue";
import { onMounted, ref, watch } from "vue";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";

const { load_modules } = use_editor_store();
const { modules } = storeToRefs(use_editor_store());

console.log("loading modules?");
load_modules();

const componentEditor = ref<InstanceType<typeof ComponentEditor>>();

watch(modules, () => {
  console.log("modules changes", modules.value);
  if (componentEditor.value) {
    for (const module of modules.value) {
      componentEditor.value.addModuleNode(module);
    }
    componentEditor.value.layout();
  }
});

onMounted(() => {
  componentEditor.value?.addModuleNode({
    exit_points: [{ name: "LoginOut", condition_script: "" }],
    insert_points: [],
    name: "Login",
    maps: [],
    max_guests: 0,
    min_guests: 0,
    resources: [],
  });
  componentEditor.value?.layout();
});

console.log("works");
</script>

<style lang="scss">
.component-editor {
  width: 100vw;
  height: 100vh;
  pointer-events: all;
}
</style>
