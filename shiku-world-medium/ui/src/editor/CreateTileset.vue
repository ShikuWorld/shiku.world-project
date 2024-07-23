<template>
  <v-card class="create-tileset" title="Create new Tileset">
    <v-text-field label="Name" v-model="tileset.name"></v-text-field>
    <v-text-field
      label="Storage Path"
      v-model="tileset.resource_path"
    ></v-text-field>
    <v-switch
      density="compact"
      label="Single image"
      v-model="is_single_image"
    ></v-switch>
    <div class="create-tileset__image" v-if="is_single_image && tileset.image">
      <v-text-field
        label="Image Path"
        class="create-tileset__image-input"
        v-model="tileset.image.path"
      ></v-text-field>
      <v-img
        :src="img"
        class="create-tileset__preview"
        ref="preview"
        @load="image_loaded"
      ></v-img>
      <div class="create-tileset__image-width-height">
        {{ tileset.image?.width }} x {{ tileset.image?.height }}
      </div>
    </div>
    <v-number-input
      label="Tile width"
      type="number"
      density="comfortable"
      :disabled="!is_single_image"
      v-model="tileset.tile_width"
    ></v-number-input>
    <v-number-input
      label="Tile height"
      density="comfortable"
      type="number"
      :disabled="!is_single_image"
      v-model="tileset.tile_height"
    ></v-number-input>
    <v-number-input
      label="Column count"
      density="comfortable"
      type="number"
      :disabled="true"
      v-model="column_count"
    ></v-number-input>
    <v-number-input
      label="Tile count"
      density="comfortable"
      type="number"
      :disabled="true"
      v-model="tile_count"
    ></v-number-input>
    <v-alert
      text="Tile width/height does not fit into image width/height!"
      type="error"
      :icon="mdiAlert"
      class="create-tileset__alert"
      v-if="tile_count_remainder !== 0"
    >
    </v-alert>
    <v-card-actions>
      <v-spacer></v-spacer>

      <v-btn text="Create Tileset" @click="save_tileset"></v-btn>
      <v-btn text="Close Dialog" @click="$emit('close')"></v-btn>
    </v-card-actions>
  </v-card>
</template>
<script lang="ts" setup>
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { computed, reactive, ref, watch } from "vue";
import { use_config_store } from "@/editor/stores/config";
import { VImg } from "vuetify/components";
import { mdiAlert } from "@mdi/js";
import { VNumberInput } from "vuetify/labs/VNumberInput";

const is_single_image = ref(true);
const tileset = reactive<Tileset>({
  columns: 0,
  resource_path: "",
  name: "",
  image: { path: "", width: 0, height: 0 },
  tile_count: 0,
  tile_height: 16,
  tiles: {},
  tile_width: 16,
  brushes: [],
});
const { resource_base_url } = use_config_store();
const img = computed(
  () => `${resource_base_url}${tileset?.image?.path ? tileset.image.path : ""}`,
);
const tile_count = computed(() =>
  tileset.image
    ? (tileset.image.width / tileset.tile_width) *
      (tileset.image.height / tileset.tile_height)
    : 0,
);
const column_count = computed(() =>
  tileset.image ? tileset.image.width / tileset.tile_width : 0,
);
const tile_count_remainder = computed(() =>
  tileset.image
    ? (tileset.image.width % tileset.tile_width) +
      (tileset.image.height % tileset.tile_height)
    : 0,
);
const preview = ref<VImg>();
function image_loaded() {
  if (preview.value?.image && tileset.image) {
    tileset.image.width = preview.value.image.naturalWidth;
    tileset.image.height = preview.value.image.naturalHeight;
  }
}
watch(is_single_image, () => {
  if (!is_single_image.value) {
    tileset.tile_count = 0;
    tileset.image = null;
  } else {
    tileset.image = { path: "", width: 0, height: 0 };
  }
});
const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", tileset: Tileset): void;
}>();
function save_tileset() {
  tileset.tile_count = tile_count.value;
  tileset.columns = column_count.value;
  emit("save", tileset);
}
</script>
<style>
.create-tileset {
  display: flex;
  flex-direction: column;
}
.create-tileset__alert {
  display: flex;
  min-height: 62px;
  width: 100%;
}
.create-tileset__image {
  display: flex;
}
.create-tileset__image-input {
  flex-grow: 1;
}
.create-tileset__preview {
  flex-grow: 0;
  width: 57px;
  height: 57px;
}
</style>
