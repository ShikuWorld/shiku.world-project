<template>
  <div class="tile-editor">
    <div class="tile-window-magnifier">
      <TilePreview :tileset="tileset" :tile_id="tile_id" />
      <CollisionEditor
        class="collision-editor"
        v-if="tile_width && tile_height && collision_shape_tmp"
        :width="tile_width"
        :height="tile_height"
        :collision_shape="collision_shape_tmp"
      ></CollisionEditor>
    </div>
    <v-select
      label="Collider"
      :hide-details="true"
      :items="collider_options"
      :model-value="collision_shape_selection"
      @update:model-value="(new_value) => update_collider(new_value)"
    ></v-select>
    <v-textarea
      :model-value="collision_data_tmp"
      @update:model-value="(new_value) => (collision_data_tmp = new_value)"
    ></v-textarea>
    <v-btn @click="update_collider_data(collision_data_tmp)">Save</v-btn>
  </div>
</template>
<script lang="ts" setup>
import { computed, onMounted, ref, toRefs, watch } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";

import { KeysOfUnion } from "@/editor/utils";
import { CollisionShape } from "@/editor/blueprints/CollisionShape";
import { tileset_key } from "@/editor/stores/editor";
import { match } from "ts-pattern";
import { use_resources_store } from "@/editor/stores/resources";
import CollisionEditor from "@/editor/editor/CollisionEditor.vue";
import TilePreview from "@/editor/editor/TilePreview.vue";

const { update_tileset_server } = use_resources_store();

const props = defineProps<{ tileset: Tileset; tile_id: number }>();
const { tileset, tile_id } = toRefs(props);
const collider_options: (KeysOfUnion<CollisionShape> | null)[] = [
  null,
  "Rectangle",
  "Circle",
  "Polygon",
];
const tile = computed(() => {
  return tileset.value.tiles[tile_id.value];
});
const collision_shape = computed(() => {
  if (tile.value) {
    return tile.value.collision_shape;
  }
  return null;
});

const collision_shape_tmp = computed(() => {
  if (collision_data_tmp.value) {
    try {
      return JSON.parse(collision_data_tmp.value);
    } catch (e) {
      return null;
    }
  }
  return null;
});

const collision_data_tmp = ref<string | null | undefined>();

watch(collision_shape, () => {
  collision_data_tmp.value = collision_data.value;
});

onMounted(() => {
  collision_data_tmp.value = collision_data.value;
});

const collision_shape_selection = computed(
  (): KeysOfUnion<CollisionShape> | null => {
    if (tile.value && tile.value.collision_shape) {
      return Object.keys(
        tile.value.collision_shape,
      )[0] as KeysOfUnion<CollisionShape>;
    }
    return null;
  },
);
const collision_data = computed(() => {
  try {
    if (collision_shape.value) {
      return JSON.stringify(collision_shape.value);
    }
  } catch (e) {
    return null;
  }
});
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

function update_collider(new_value: KeysOfUnion<CollisionShape> | null) {
  if (tileset.value && tile_id.value != undefined) {
    if (collision_shape_selection.value === new_value) {
      return;
    }
    if (new_value === null) {
      update_tileset_server(tileset_key(tileset.value), {
        RemoveCollisionShape: tile_id.value,
      });
      return;
    }

    update_tileset_server(tileset_key(tileset.value), {
      UpdateCollisionShape: [tile_id.value, create_collision_shape(new_value)],
    });
  }
}

function update_collider_data(new_value: string | null | undefined) {
  if (!new_value) {
    return;
  }

  try {
    const collision_shape = JSON.parse(new_value) as CollisionShape;
    const key = Object.keys(
      collision_shape,
    )[0] as KeysOfUnion<CollisionShape> | null;
    if (collider_options.includes(key)) {
      update_tileset_server(tileset_key(tileset.value), {
        UpdateCollisionShape: [tile_id.value, collision_shape],
      });
    }
  } catch (e) {
    return;
  }
}

function create_collision_shape(
  kind: KeysOfUnion<CollisionShape>,
): CollisionShape {
  return match(kind)
    .with(
      "Polygon",
      (): CollisionShape => ({
        Polygon: [
          [0, 0],
          [0, 8],
          [8, 8],
        ],
      }),
    )
    .with("Circle", (): CollisionShape => ({ Circle: [8, 8, 8] }))
    .with("Rectangle", (): CollisionShape => ({ Rectangle: [0, 0, 8, 8] }))
    .exhaustive();
}
</script>
<style>
.tile-window-magnifier {
  width: 100%;
  margin: 64px 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transform: scale(4);
}
.collision-editor {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
