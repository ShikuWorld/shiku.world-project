<template>
  <div class="tile-window" :style="style_window">
    <div class="tile-image" :style="style_tile"></div>
  </div>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { get_resource_url } from "@/client/config/config";
import { Tileset } from "@/editor/blueprints/Tileset";

interface Props {
  tileset: Tileset;
  tile_id: number;
}

const props = defineProps<Props>();

const { tileset, tile_id } = toRefs(props);

const tile = computed(() => {
  return tileset.value.tiles[tile_id.value];
});
const rows = computed(() => {
  if (tileset.value.image) {
    return tileset.value.image.width / tileset.value.tile_width;
  }
  return 0;
});
function get_x_y_from_gid(gId: number, rows: number): [number, number] {
  if (tile.value?.image) {
    return [0, 0];
  }
  const y = Math.floor((gId - 1) / rows) + 1;
  const x = ((gId - 1) % rows) + 1;
  return [y, x];
}
const tile_width = computed(() => {
  return tileset.value.image
    ? tileset.value.tile_width
    : tile.value.image?.width;
});
const tile_height = computed(() => {
  return tileset.value.image
    ? tileset.value.tile_height
    : tile.value.image?.height;
});
const style_window = computed(() => {
  return {
    width: `${tile_width.value}px`,
    height: `${tile_height.value}px`,
  };
});
const style_tile = computed(() => {
  const background_image = tileset.value.image
    ? `url(${get_resource_url()}/${tileset.value.image.path})`
    : `url(${get_resource_url()}/${tile.value?.image?.path})`;
  const [y, x] = get_x_y_from_gid(tile_id.value, rows.value);
  return {
    width: "100%",
    height: "100%",
    backgroundImage: background_image,
    backgroundPositionX: `${
      tileset.value.image ? -tileset.value.tile_width * (x - 1) : 0
    }px`,
    backgroundPositionY: `${
      tileset.value.image ? -tileset.value.tile_height * (y - 1) : 0
    }px`,
  };
});
</script>
