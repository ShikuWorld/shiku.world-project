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
    game_node: { selected_game_node_id?: string; path: number[] };
    nothing: undefined;
  };
}

export const use_inspector_store = defineStore("inspector", {
  state: (): InspectorStore => ({
    active_component: "nothing",
    component_stores: {
      game_node: { path: [] },
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
    select_game_node(game_node_id: string, path: number[]) {
      this.component_stores.game_node.selected_game_node_id = game_node_id;
      this.component_stores.game_node.path = path;
      this.active_component = "game_node";
    },
  },
});
