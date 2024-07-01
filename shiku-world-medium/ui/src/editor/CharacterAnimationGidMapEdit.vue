<template>
  <v-list>
    <v-list-item
      lines="two"
      density="compact"
      v-for="direction in current_directions"
    >
      <v-select
        :model-value="direction"
        :items="available_direction_options"
        density="compact"
        hide-details="auto"
        @update:model-value="
          (new_value) =>
            emit(
              'setGidMapValue',
              new_value,
              frame.gid_map[direction],
              direction,
            )
        "
      >
        <template v-slot:append>
          <v-icon
            :icon="mdiTrashCan"
            @click="emit('removeGidEntry', direction)"
          ></v-icon>
        </template>
      </v-select>
      <v-number-input
        :reverse="false"
        controlVariant="stacked"
        density="compact"
        :hide-details="true"
        variant="outlined"
        :step="1"
        :model-value="frame.gid_map[direction]"
        @update:model-value="
          (new_value) => emit('setGidMapValue', direction, new_value)
        "
      >
        <template v-slot:append>
          <TilePreview :tileset="tileset" :tile_id="frame.gid_map[direction]" />
        </template>
      </v-number-input>
    </v-list-item>
  </v-list>
  <v-btn :icon="mdiPlus" @click="on_gid_add_click"></v-btn>
</template>
<script setup lang="ts">
import { CharacterDirection } from "@/editor/blueprints/CharacterDirection";
import { mdiPlus, mdiTrashCan } from "@mdi/js";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { CharacterAnimationFrame } from "@/editor/blueprints/CharacterAnimationFrame";
import { computed, toRefs } from "vue";
import { Tileset } from "@/editor/blueprints/Tileset";
import TilePreview from "@/editor/editor/TilePreview.vue";

interface Props {
  tileset: Tileset;
  character_animation_directions: CharacterDirection[];
  frame: CharacterAnimationFrame;
}

const props = defineProps<Props>();

const { frame, character_animation_directions } = toRefs(props);

const emit = defineEmits<{
  (
    e: "setGidMapValue",
    direction: CharacterDirection,
    gid: number,
    prev_direction?: CharacterDirection,
  ): void;
  (e: "removeGidEntry", direction: CharacterDirection): void;
}>();

const current_directions = computed(() => {
  return character_animation_directions.value.filter(
    (direction) => frame.value.gid_map[direction] !== undefined,
  );
});

const available_direction_options = computed(() => {
  return character_animation_directions.value.filter(
    (direction) => frame.value.gid_map[direction] === undefined,
  );
});

const on_gid_add_click = () => {
  if (available_direction_options.value.length > 0) {
    emit("setGidMapValue", available_direction_options.value[0], 0);
  }
};
</script>
