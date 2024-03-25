import { defineStore } from "pinia";

export type InspectorComponent =
  | "module"
  | "tile"
  | "map"
  | "game_node"
  | "nothing";

export interface InspectorStore {
  active_component: InspectorComponent;
  component_stores: {
    module: string;
    tile: string;
    map: string;
    game_node: {
      scene_resource_path?: string;
      selected_game_node_id?: string;
      selection_path?: number[];
    };
    nothing: undefined;
  };
}

export const use_inspector_store = defineStore("inspector", {
  state: (): InspectorStore => ({
    active_component: "nothing",
    component_stores: {
      game_node: {},
      map: "",
      module: "",
      nothing: undefined,
      tile: "",
    },
  }),
  actions: {
    set_inspector_component(component: InspectorComponent) {
      this.active_component = component;
    },
    select_game_node(
      scene_resource_path: string,
      game_node_id: string,
      path: number[],
    ) {
      this.component_stores = {
        ...this.component_stores,
        game_node: {
          scene_resource_path: scene_resource_path,
          selected_game_node_id: game_node_id,
          selection_path: path,
        },
      };
      this.active_component = "game_node";
    },
  },
});
