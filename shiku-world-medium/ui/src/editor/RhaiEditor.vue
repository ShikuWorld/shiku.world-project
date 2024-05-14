<script setup lang="ts">
import { computed, ref, toRefs } from "vue";
import { EditorView, basicSetup } from "codemirror";
import { rust } from "@codemirror/lang-rust";
import { cobalt } from "thememirror";
import { use_resources_store } from "@/editor/stores/resources";
import { storeToRefs } from "pinia";

const props = defineProps<{
  script_resource_path: string;
}>();
const { script_resource_path } = toRefs(props);
const { get_or_load_script, update_script_server } = use_resources_store();
const { script_map } = storeToRefs(use_resources_store());
const script = computed(() =>
  get_or_load_script(script_map.value, script_resource_path.value),
);

const editor = ref<HTMLElement>();
const editor_view = ref<EditorView | undefined>();
const is_active = ref<boolean>(false);
function open() {
  is_active.value = true;

  setTimeout(() => {
    if (editor.value && script.value) {
      editor_view.value = new EditorView({
        doc: script.value.content,
        extensions: [basicSetup, rust(), cobalt],
        parent: editor.value,
      });
    }
  }, 100);
}
defineExpose({ open });
function on_save() {
  if (editor_view.value && script.value) {
    update_script_server({
      id: script.value.id,
      name: script.value.name,
      resource_path: script.value.resource_path,
      content: editor_view.value.state.doc.toString(),
    });
  }
}
</script>

<template>
  <v-dialog max-width="800" v-model="is_active" :scrim="'#ffffff'">
    <v-card v-if="!script">Loading...</v-card>
    <v-card v-if="script" title="Lets cooode">
      <div ref="editor"></div>
      <v-card-actions>
        <v-spacer></v-spacer>

        <v-btn text="Save" @click="on_save"></v-btn>
        <v-btn text="Close Dialog" @click="is_active = false"></v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped></style>
