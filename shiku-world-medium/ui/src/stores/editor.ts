import { defineStore } from "pinia";
import { Conductor } from "@/editor/blueprints/Conductor";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { Isometry } from "@/client/entities";

export type Point = { y: number; x: number };

export interface EditorStore {
  editor_open: boolean;
  main_door_status: boolean;
  selected_module_id: string;
  current_main_instance_id: string;
  current_map_path: string;
  edit_module_id: string;
  conductor: Conductor;
  tile_brush: number[][];
  current_map_index: number;
  module_instance_map: { [module_id: string]: string[] };
  selected_resource_tab: number;
  open_resource_paths: string[];
  selected_tileset_path: string;
  selected_tile_id: number;
  selected_tile_position: Point;
  selected_scene_props: { scene_path: string | null; transparency: number };
  current_main_instance: { instance_id?: string; world_id?: string };
}

export const use_editor_store = defineStore("editor", {
  state: (): EditorStore => ({
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
    edit_module_id: "",
    current_map_index: 0,
    current_main_instance: {},
    selected_tile_position: { x: 0, y: 0 },
    conductor: { module_connection_map: {}, resources: [], gid_map: [] },
    selected_scene_props: { scene_path: null, transparency: 0.5 },
  }),
  actions: {
    set_tile_brush(brush: number[][]) {
      this.tile_brush = brush;
    },
    select_tile_position(tile_position: { x: number; y: number }) {
      this.selected_tile_position = tile_position;
    },
    set_camera_position(instance_id: string, world_id: string, iso: Isometry) {
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
      this.module_instance_map = module_instance_map;
    },
    set_current_main_instance(instance_id: string, world_id: string) {
      this.current_main_instance = { instance_id, world_id };

      window.medium.swap_main_render_instance(instance_id, world_id);
    },
    set_selected_scene(scene_path: string) {
      this.selected_scene_props.scene_path = scene_path;
    },
    add_module_instance(module_id: string, game_instance_id: string) {
      this.module_instance_map = {
        ...this.module_instance_map,
        [module_id]: this.module_instance_map[module_id]
          ? [...this.module_instance_map[module_id], game_instance_id]
          : [],
      };
    },
    remove_module_instance(module_id: string, game_instance_id: string) {
      this.module_instance_map = {
        ...this.module_instance_map,
        [module_id]: this.module_instance_map[module_id]
          ? this.module_instance_map[module_id].filter(
              (g) => g != game_instance_id,
            )
          : [],
      };
    },
    set_selected_tile(tileset_path: string, tile_id: number) {
      this.selected_tileset_path = tileset_path;
      this.selected_tile_id = tile_id;
    },
    set_selected_resource_tab(index: number) {
      this.selected_resource_tab = index;
    },
    close_resource(path: string) {
      this.open_resource_paths = this.open_resource_paths.filter(
        (p) => p !== path,
      );
    },
    add_open_resource_path(path: string): number {
      if (this.open_resource_paths.includes(path)) {
        return this.open_resource_paths.findIndex((p) => p === path);
      }
      this.open_resource_paths = [...this.open_resource_paths, path];
      return this.open_resource_paths.length - 1;
    },
    set_selected_module_id(id: string) {
      this.selected_module_id = id;
    },
    set_current_main_instance_id(id: string) {
      this.current_main_instance_id = id;
    },
    set_main_door_status(status: boolean) {
      this.main_door_status = status;
    },
    set_conductor(conductor: Conductor) {
      this.conductor = conductor;
    },
    open_game_instance_server(module_id: string) {
      send_admin_event({ OpenInstance: module_id });
    },
    start_inspecting_world(
      module_id: string,
      game_instance_id: string,
      world_id: string,
    ) {
      send_admin_event({
        StartInspectingWorld: [module_id, game_instance_id, world_id],
      });
    },
    stop_inspecting_world(
      module_id: string,
      game_instance_id: string,
      world_id: string,
    ) {
      send_admin_event({
        StopInspectingWorld: [module_id, game_instance_id, world_id],
      });
    },
    save_conductor_server(conductor: Conductor) {
      send_admin_event({ UpdateConductor: conductor });
    },
    show_editor() {
      this.editor_open = true;
    },
    hide_editor() {
      this.editor_open = false;
    },
  },
});

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
