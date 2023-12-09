<template>
  <div class="tile-selector">
    <v-tabs v-model="tab" bg-color="primary">
      <v-tab
        v-for="tileset in tilesets"
        :key="tileset_key(tileset)"
        :value="tileset_key(tileset)"
        >{{ tileset.name }}</v-tab
      >
    </v-tabs>
    <v-window v-model="tab">
      <v-window-item
        v-for="tileset in tilesets"
        :key="tileset_key(tileset)"
        :value="tileset_key(tileset)"
      >
        {{ tileset.name }}
      </v-window-item>
    </v-window>
  </div>
</template>
<script lang="ts" setup>
import { onMounted, ref, toRefs, watch } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import { tileset_key } from "@/editor/stores/editor";
const tab = ref<string>();

const props = defineProps<{ tilesets: Tileset[] }>();
const { tilesets } = toRefs(props);
onMounted(() => {
  if (tilesets.value.length > 0) {
    tab.value = tileset_key(props.tilesets[0]);
  }
});

watch(tilesets, () => {
  if (!tab.value && tilesets.value.length > 0) {
    tab.value = tileset_key(props.tilesets[0]);
  }
});
</script>
<style></style>
