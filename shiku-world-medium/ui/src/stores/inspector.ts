import { defineStore } from "pinia";
import { Entity } from "@/editor/blueprints/Entity";

export interface InspectorStore {
  component_stores: {
    module: string;
    tile: string;
    map: string;
    game_node: {
      scene_resource_path?: string;
      selected_game_node_id?: string;
      selected_entity_id?: Entity | null;
      selection_path?: number[];
      is_instance?: boolean;
    };
    character_animation: {
      character_animation_resource_path?: string;
      selected_state_id?: number;
    };
    nothing: undefined;
  };
}

export const use_inspector_store = defineStore("inspector", {
  state: (): InspectorStore => ({
    component_stores: {
      game_node: {},
      map: "",
      module: "",
      character_animation: {},
      nothing: undefined,
      tile: "",
    },
  }),
  actions: {
    select_character_animation_state(
      selected_state_id: number,
      character_animation_resource_path: string,
    ) {
      this.component_stores = {
        ...this.component_stores,
        character_animation: {
          selected_state_id,
          character_animation_resource_path,
        },
      };
    },
    select_game_node(
      scene_resource_path: string,
      selected_game_node_id: string,
      selection_path: number[],
      is_instance: boolean,
      selected_entity_id: Entity | null,
    ) {
      this.component_stores = {
        ...this.component_stores,
        game_node: {
          scene_resource_path,
          selected_game_node_id,
          selection_path,
          is_instance,
          selected_entity_id,
        },
      };
    },
  },
});
