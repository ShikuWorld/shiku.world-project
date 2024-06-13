<template>
  <v-list>
    <v-list-item
      v-for="(instance_id, index) in module_instances"
      :key="instance_id"
      :title="`${module.name}-${index}`"
    >
      <v-list>
        <v-list-item
          v-for="game_map in game_maps"
          :prepend-icon="get_icon(instance_id, game_map.world_id)"
          @click="
            emit(
              'world_click',
              module.id,
              instance_id,
              game_map.world_id,
              map_key(game_map),
            )
          "
          >{{ game_map.name }}
        </v-list-item>
      </v-list>
    </v-list-item>
  </v-list>
</template>
<script lang="ts" setup>
import { Module } from "@/editor/blueprints/Module";
import { computed, toRefs } from "vue";
import { GameMap } from "@/editor/blueprints/GameMap";
import { storeToRefs } from "pinia";
import { map_key, use_editor_store } from "@/editor/stores/editor";
import { mdiAccessPoint, mdiAccessPointOff, mdiCheckBold } from "@mdi/js";
import { use_resources_store } from "@/editor/stores/resources";
import { use_game_instances_store } from "@/editor/stores/game-instances";

const { current_main_instance } = storeToRefs(use_editor_store());
const { game_instance_exists } = use_game_instances_store();
const { game_map_map } = storeToRefs(use_resources_store());

const props = defineProps<{
  module: Module;
  module_instances: string[];
  show_current_instance?: boolean;
}>();
const { module, module_instances, show_current_instance } = toRefs(props);
const emit = defineEmits<{
  (
    e: "world_click",
    module_is: string,
    instance_id: string,
    world_id: string,
    map_resource_path: string,
  ): void;
}>();

function get_icon(instance_id: string, world_id: string) {
  if (show_current_instance.value) {
    return current_main_instance.value.instance_id === instance_id &&
      current_main_instance.value.world_id === world_id
      ? mdiCheckBold
      : "";
  }
  return game_instance_exists(instance_id, world_id)
    ? mdiAccessPoint
    : mdiAccessPointOff;
}

const game_maps = computed<GameMap[]>(() =>
  module.value.resources
    .filter((m) => m.kind === "Map")
    .map((m) => game_map_map.value[m.path])
    .filter((m) => !!m),
);
</script>
