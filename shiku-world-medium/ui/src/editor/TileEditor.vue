<template>
  <div class="tile-editor">{{ tile_id }}{{ tileset.name }}</div>
</template>
<script lang="ts" setup>
import { toRefs } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";

const props = defineProps<{ tileset: Tileset; tile_id: number }>();
const { tileset, tile_id } = toRefs(props);
</script>
<style></style>
