import { defineStore } from "pinia";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { Isometry } from "@/client/entities";
import { reactive, toRefs, watch } from "vue";
import { use_game_instances_store } from "@/editor/stores/game-instances";
import { use_resources_store } from "@/editor/stores/resources";
import { Entity } from "@/editor/blueprints/Entity";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { LayerKind } from "@/editor/blueprints/LayerKind";

export type Point = { y: number; x: number };

export type InspectorComponent =
  | "module"
  | "tile"
  | "map"
  | "game_node"
  | "character_animation_state"
  | "nothing";

export interface EditorStore {
  editor_open: boolean;
  main_door_status: boolean;
  selected_module_id: string;
  current_main_instance_id: string;
  current_map_path: string;
  edit_module_id: string;
  tile_brush: number[][];
  current_map_index: number;
  module_instance_map: { [module_id: string]: string[] };
  selected_nav_top_tab: "current" | "modules" | "resources";
  selected_resource_tab: number;
  open_resource_paths: string[];
  selected_tileset_path: string;
  selected_tile_id: number;
  selected_tile_position: Point;
  selected_tile_layer: LayerKind;
  client_connection_ready: boolean;
  selected_scene_props: {
    scene_path: string | null;
    transparency: number;
    is_pinned: boolean;
  };
  inspecting_worlds: {
    main?: {
      module_id: string;
      instance_id: string;
      world_id: string;
      map_resource_path: string;
    };
  };
  current_main_instance: { instance_id?: string; world_id?: string };
  active_component: InspectorComponent;
}

export const use_editor_store = defineStore(
  "editor",
  () => {
    const state: EditorStore = reactive({
      inspecting_worlds: {},
      active_component: "nothing",
      editor_open: false,
      module_instance_map: {},
      main_door_status: false,
      tile_brush: [[0]],
      selected_module_id: "",
      selected_tileset_path: "",
      selected_tile_id: 0,
      current_main_instance_id: "",
      current_map_path: "",
      open_resource_paths: [],
      selected_resource_tab: 0,
      selected_nav_top_tab: "current",
      edit_module_id: "",
      client_connection_ready: false,
      selected_tile_layer: "Terrain",
      current_map_index: 0,
      current_main_instance: {},
      selected_tile_position: { x: 0, y: 0 },
      selected_scene_props: {
        scene_path: null,
        transparency: 0.5,
        is_pinned: false,
      },
    });

    setInterval(() => {
      if (
        window?.medium?.communication_state?.is_connection_ready &&
        !state.client_connection_ready
      ) {
        state.client_connection_ready = true;
      } else if (
        !window?.medium?.communication_state?.is_connection_ready &&
        state.client_connection_ready
      ) {
        state.client_connection_ready = false;
      }
    }, 100);

    const actions = {
      set_inspector_component(component: InspectorComponent) {
        state.active_component = component;
      },
      set_selected_tile_layer(layer: LayerKind) {
        state.selected_tile_layer = layer;
      },
      set_tile_brush(brush: number[][]) {
        state.tile_brush = brush;
        if (
          state.current_main_instance.instance_id &&
          state.current_main_instance.world_id
        ) {
          window.medium.adjust_brush_hover(
            state.current_main_instance.instance_id,
            state.current_main_instance.world_id,
            brush,
          );
        }
      },
      set_selected_nav_top_tab(index: EditorStore["selected_nav_top_tab"]) {
        state.selected_nav_top_tab = index;
      },
      select_tile_position(tile_position: { x: number; y: number }) {
        state.selected_tile_position = tile_position;
      },
      set_camera_position(
        instance_id: string,
        world_id: string,
        iso: Isometry,
      ) {
        window.medium.set_camera_iso(instance_id, world_id, iso);
      },
      set_camera_zoom(instance_id: string, world_id: string, zoom: number) {
        window.medium.set_camera_zoom(instance_id, world_id, zoom);
      },
      get_camera_position(instance_id: string, world_id: string): Isometry {
        return window.medium.get_camera_iso(instance_id, world_id);
      },
      get_camera_zoom(instance_id: string, world_id: string): number {
        return window.medium.get_camera_zoom(instance_id, world_id);
      },
      set_game_instance_map(instance_data: [string, string[]][]) {
        const module_instance_map: EditorStore["module_instance_map"] = {};
        for (const d of instance_data) {
          module_instance_map[d[0]] = d[1];
        }
        state.module_instance_map = module_instance_map;
      },
      set_current_main_instance(instance_id: string, world_id: string) {
        state.current_main_instance = { instance_id, world_id };

        window.medium.swap_main_render_instance(instance_id, world_id);
      },
      set_selected_scene(scene_path: string) {
        state.selected_scene_props.scene_path = scene_path;
      },
      add_module_instance(module_id: string, game_instance_id: string) {
        state.module_instance_map = {
          ...state.module_instance_map,
          [module_id]: state.module_instance_map[module_id]
            ? [...state.module_instance_map[module_id], game_instance_id]
            : [],
        };
      },
      remove_module_instance(module_id: string, game_instance_id: string) {
        state.module_instance_map = {
          ...state.module_instance_map,
          [module_id]: state.module_instance_map[module_id]
            ? state.module_instance_map[module_id].filter(
                (g) => g != game_instance_id,
              )
            : [],
        };
      },
      set_selected_tile(tileset_path: string, tile_id: number) {
        state.selected_tileset_path = tileset_path;
        state.selected_tile_id = tile_id;
      },
      set_selected_resource_tab(index: number) {
        state.selected_resource_tab = index;
      },
      close_resource(path: string) {
        state.open_resource_paths = state.open_resource_paths.filter(
          (p) => p !== path,
        );
      },
      add_open_resource_path(path: string): number {
        if (state.open_resource_paths.includes(path)) {
          return state.open_resource_paths.findIndex((p) => p === path);
        }
        state.open_resource_paths = [...state.open_resource_paths, path];
        return state.open_resource_paths.length - 1;
      },
      set_selected_module_id(id: string) {
        state.selected_module_id = id;
      },
      set_current_main_instance_id(id: string) {
        state.current_main_instance_id = id;
      },
      set_main_door_status(status: boolean) {
        state.main_door_status = status;
      },

      open_game_instance_server(module_id: string) {
        send_admin_event({ OpenInstance: module_id });
      },
      remove_entity_server(
        module_id: string,
        game_instance_id: string,
        world_id: string,
        entity: Entity,
      ) {
        send_admin_event({
          RemoveInstanceNode: [module_id, game_instance_id, world_id, entity],
        });
      },
      add_entity_server(
        module_id: string,
        game_instance_id: string,
        world_id: string,
        parent_entity_id: Entity,
        game_node: GameNodeKind,
      ) {
        send_admin_event({
          AddNodeToInstanceNode: [
            module_id,
            game_instance_id,
            world_id,
            parent_entity_id,
            game_node,
          ],
        });
      },
      start_inspecting_world(
        module_id: string,
        game_instance_id: string,
        world_id: string,
        map_resource_path: string,
      ) {
        send_admin_event({
          StartInspectingWorld: [module_id, game_instance_id, world_id],
        });
        state.inspecting_worlds.main = {
          module_id,
          instance_id: game_instance_id,
          world_id,
          map_resource_path,
        };
      },
      stop_inspecting_world(
        module_id: string,
        game_instance_id: string,
        world_id: string,
      ) {
        send_admin_event({
          StopInspectingWorld: [module_id, game_instance_id, world_id],
        });
        delete state.inspecting_worlds.main;
      },
      reset_world(module_id: string, instance_id: string, world_id: string) {
        send_admin_event({
          ResetGameWorld: [module_id, instance_id, world_id],
        });
      },
      show_editor() {
        state.editor_open = true;
      },
      hide_editor() {
        state.editor_open = false;
      },
    };

    const { get_or_load_scene } = use_resources_store();
    const { scene_map } = toRefs(use_resources_store());
    watch(
      [state.selected_scene_props, scene_map, state.client_connection_ready],
      () => {
        if (!state.client_connection_ready) {
          return;
        }
        const scene_path = state.selected_scene_props.scene_path;
        if (!scene_path) {
          return;
        }
        const scene = get_or_load_scene(scene_map.value, scene_path);
        if (!scene) {
          return;
        }
        const { set_and_render_blueprint_render } = use_game_instances_store();
        if (
          state.inspecting_worlds?.main?.instance_id &&
          state.inspecting_worlds?.main?.world_id &&
          window.medium.is_instance_ready(
            state.inspecting_worlds.main.instance_id,
            state.inspecting_worlds.main.world_id,
          )
        ) {
          set_and_render_blueprint_render(
            state.selected_module_id,
            scene_path,
            scene,
            state.selected_scene_props.is_pinned,
          );
        }
      },
    );

    return {
      ...toRefs(state),
      ...actions,
    };
  },
  {
    persist: {
      afterRestore: (ctx) => {
        ctx.store.$state.editor_open = false;
      },
    },
  },
);
function send_admin_event(event: AdminToSystemEvent) {
  if (window.medium.communication_state.is_connection_open) {
    window.medium.communication_state.ws_connection.send(JSON.stringify(event));
  }
}

export function tileset_key(tileset: Tileset) {
  return `${tileset.resource_path}/${tileset.name}.tileset.json`;
}

export function resource_key(resource: BlueprintResource) {
  return `${resource.dir}/${resource.file_name}`;
}

export function map_key(game_map: { resource_path: string; name: string }) {
  return `${game_map.resource_path}/${game_map.name}.map.json`;
}
