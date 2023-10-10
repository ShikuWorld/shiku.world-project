<template>
  <div
    class="image-hover-map"
    v-for="image in config.images"
    :style="{ height: `${config.height}px` }"
  >
    <v-tooltip
      :text="
        get_active_hover_text(
          compute_input_value(image.state_value, context),
          image
        )
      "
      location="top"
    >
      <template v-slot:activator="{ props }">
        <img
          class="image-hover-map__image"
          v-bind="props"
          :style="{ left: `${image.pos_x}px`, top: `${image.pos_y}px` }"
          :alt="
            get_active_hover_text(
              compute_input_value(image.state_value, context),
              image
            )
          "
          :src="
            get_active_image_src(
              compute_input_value(image.state_value, context),
              image
            )
          "
        />
      </template>
    </v-tooltip>
  </div>
</template>

<style lang="scss">
.image-hover-map {
  width: 100%;
  position: relative;

  &__image {
    position: absolute;
    pointer-events: all;
    cursor: pointer;
  }
}
</style>

<script lang="ts" setup>
import { DataContext, ExtractionObject, use_layout_functions } from "@/ui";

interface ImageState {
  value: number | string | boolean;
  hover_text: string;
  image_src: string;
}
interface Image {
  default_hover_text: string;
  default_image_src: string;
  pos_x: number;
  pos_y: number;
  state_value: ExtractionObject;
  states: ImageState[];
}

export interface ImageHoverMap {
  name: "ImageHoverMap";
  config: {
    height: number;
    images: Image[];
  };
  context: DataContext;
}

const { compute_input_value } = use_layout_functions();

defineProps<ImageHoverMap>();

const get_active_image_src = (
  state_value: number | string | boolean,
  image: Image
) =>
  image.states.find((state) => state_value === state.value)?.image_src ||
  image.default_image_src;

const get_active_hover_text = (
  state_value: number | string | boolean,
  image: Image
) =>
  image.states.find((state) => state_value === state.value)?.hover_text ||
  image.default_hover_text;
</script>
