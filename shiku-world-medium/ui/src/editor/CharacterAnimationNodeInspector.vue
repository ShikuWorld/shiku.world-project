<template>
  <v-text-field
    label="name"
    :model-value="character_animation.name"
    v-on:change="change_name"
    density="compact"
    hide-details="auto"
  ></v-text-field>
  <v-select
    label="Start Direction"
    :model-value="character_animation.start_direction"
    :items="character_animation_directions"
    density="compact"
    hide-details="auto"
    @update:model-value="change_current_direction"
  ></v-select>
  <v-select
    label="Start State"
    :model-value="character_animation.start_state"
    :items="state_options"
    item-value="value"
    item-title="label"
    density="compact"
    hide-details="auto"
    @update:model-value="change_current_state"
  ></v-select>
  <div>
    <h4 class="header">States</h4>
    <v-list lines="one">
      <v-list-item
        density="compact"
        v-for="(state, key) in character_animation.states"
        :key="key"
      >
        {{ state.name }}
        <template v-slot:append>
          <v-icon :icon="mdiTrashCan" @click="remove_state(key)"></v-icon>
        </template>
      </v-list-item>
    </v-list>
    <v-text-field
      label="State name"
      v-model="new_state_name"
      density="compact"
      hide-details="auto"
    >
      <template v-slot:append>
        <v-icon :icon="mdiPlus" @click="add_state"></v-icon>
      </template>
    </v-text-field>
  </div>
  <div v-if="selected_state" class="state-editor">
    <h4>{{ selected_state.name }}</h4>
    <v-switch
      label="Loop Animation"
      v-model="selected_state.loop_animation"
      @update:model-value="(new_value) => set_loop_animation(new_value)"
    ></v-switch>
    <v-virtual-scroll :items="selected_state.frames" :height="300">
      <template v-slot:default="{ item, index }">
        <v-number-input
          :reverse="false"
          controlVariant="stacked"
          :hideInput="false"
          :inset="false"
          density="compact"
          :hide-details="true"
          variant="outlined"
          :step="1"
          :model-value="item.duration_in_ms"
          @update:model-value="
            (new_value) => set_frame_duration(index, new_value)
          "
        >
          <template v-slot:append>
            <v-icon
              :icon="mdiTrashCan"
              @click="remove_frame(index)"
            ></v-icon> </template
        ></v-number-input>
        <div class="gids" v-if="character_animation_tileset">
          <CharacterAnimationGidMapEdit
            :character_animation_directions="character_animation_directions"
            :frame="item"
            :tileset="character_animation_tileset"
            @setGidMapValue="
              (direction, gid, prev_direction) =>
                set_gid_map_entry(index, direction, gid, prev_direction)
            "
            @removeGidEntry="
              (direction) => remove_gid_map_entry(index, direction)
            "
          ></CharacterAnimationGidMapEdit>
        </div>
      </template>
    </v-virtual-scroll>
    <v-btn :icon="mdiPlus" @click="add_frame"></v-btn>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, toRefs } from "vue";
import { CharacterAnimation } from "@/editor/blueprints/CharacterAnimation";
import { CharacterDirection } from "@/editor/blueprints/CharacterDirection";
import { use_resources_store } from "@/editor/stores/resources";
import { mdiPlus, mdiTrashCan } from "@mdi/js";
import { VNumberInput } from "vuetify/labs/VNumberInput";
import { CharacterAnimationFrame } from "@/editor/blueprints/CharacterAnimationFrame";
import CharacterAnimationGidMapEdit from "@/editor/editor/CharacterAnimationGidMapEdit.vue";
import { storeToRefs } from "pinia";

interface Props {
  character_animation: CharacterAnimation;
  selected_state_id?: number;
}

const { get_or_load_tileset } = use_resources_store();

const { tileset_map } = storeToRefs(use_resources_store());

const character_animation_tileset = computed(() =>
  get_or_load_tileset(
    tileset_map.value,
    character_animation.value.tileset_resource,
  ),
);

const set_loop_animation = (new_value: boolean | null) => {
  if (new_value === null) {
    return;
  }
  if (selected_state.value) {
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [selected_state_id.value as number]: {
          ...selected_state.value,
          loop_animation: new_value,
        },
      },
    });
  }
};

const set_frame_duration = (index: number, new_value: number) => {
  if (selected_state.value) {
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [selected_state_id.value as number]: {
          ...selected_state.value,
          frames: selected_state.value.frames.map((frame, i) =>
            i === index ? { ...frame, duration_in_ms: new_value } : frame,
          ),
        },
      },
    });
  }
};

const remove_gid_map_entry = (
  frame_index: number,
  direction: CharacterDirection,
) => {
  console.log("remove_gid_map_entry", frame_index, direction);
  if (selected_state.value) {
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [selected_state_id.value as number]: {
          ...selected_state.value,
          frames: selected_state.value.frames.map((frame, i) =>
            i === frame_index
              ? {
                  ...frame,
                  gid_map: {
                    ...frame.gid_map,
                    [direction]: undefined,
                  },
                }
              : frame,
          ),
        },
      },
    });
  }
};

const set_gid_map_entry = (
  frame_index: number,
  direction: CharacterDirection,
  gid: number,
  prev_direction?: CharacterDirection,
) => {
  if (selected_state.value)
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [selected_state_id.value as number]: {
          ...selected_state.value,
          frames: selected_state.value.frames.map((frame, i) =>
            i === frame_index
              ? {
                  ...frame,
                  gid_map: {
                    ...frame.gid_map,
                    ...(prev_direction && direction != prev_direction
                      ? { [prev_direction]: undefined }
                      : {}),
                    [direction]: gid,
                  },
                }
              : frame,
          ),
        },
      },
    });
};

const remove_frame = (index: number) => {
  if (selected_state.value) {
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [selected_state_id.value as number]: {
          ...selected_state.value,
          frames: selected_state.value.frames.filter((_, i) => i !== index),
        },
      },
    });
  }
};

const add_frame = () => {
  if (selected_state.value) {
    const last_frame_ms =
      selected_state.value.frames[selected_state.value.frames.length - 1]
        ?.duration_in_ms ?? 100;
    const frame = {
      duration_in_ms: last_frame_ms,
      gid_map: {},
    } as CharacterAnimationFrame;
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [selected_state_id.value as number]: {
          ...selected_state.value,
          frames: [...selected_state.value.frames, frame],
        },
      },
    });
  }
};

const props = defineProps<Props>();

const { character_animation, selected_state_id } = toRefs(props);
const { save_character_animation_server } = use_resources_store();

const new_state_name = ref("");

const character_animation_directions: CharacterDirection[] = [
  "Down",
  "Left",
  "Right",
  "Up",
];

const state_options = computed(() => {
  return Object.entries(character_animation.value.states).map(
    ([key, state]) => ({
      label: state.name,
      value: Number(key),
    }),
  ) as { label: string; value: number }[];
});

const selected_state = computed(() => {
  if (character_animation.value.states[selected_state_id.value as number]) {
    return character_animation.value.states[selected_state_id.value as number];
  }
  return undefined;
});

const change_name = (new_name: string) => {
  save_character_animation_server({
    ...character_animation.value,
    name: new_name,
  });
};

const change_current_direction = (new_direction: CharacterDirection) => {
  save_character_animation_server({
    ...character_animation.value,
    start_direction: new_direction,
  });
};

const change_current_state = (new_state_id: number) => {
  save_character_animation_server({
    ...character_animation.value,
    start_state: new_state_id,
  });
};

const next_state_key = computed(() => {
  return (
    Math.max(
      ...Object.keys(character_animation.value.states).map((key) =>
        Number(key),
      ),
    ) + 1
  );
});

const add_state = () => {
  if (new_state_name.value && new_state_name.value.trim().length > 0) {
    save_character_animation_server({
      ...character_animation.value,
      states: {
        ...character_animation.value.states,
        [next_state_key.value]: {
          name: new_state_name.value,
          frames: [],
          loop_animation: false,
        },
      },
    });
  }
};

const remove_state = (key: number) => {
  save_character_animation_server({
    ...character_animation.value,
    states: Object.fromEntries(
      Object.entries(character_animation.value.states).filter(
        ([k]) => Number(k) !== Number(key),
      ),
    ),
  });
};
</script>

<style scoped>
.header {
  padding: 8px 16px;
}
.state-editor {
  padding: 16px 0 0 16px;
}
</style>
