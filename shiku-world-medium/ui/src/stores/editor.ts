import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";
import { ModuleUpdate } from "@/editor/blueprints/ModuleUpdate";
import { Conductor } from "@/editor/blueprints/Conductor";
import { FileBrowserResult } from "@/editor/blueprints/FileBrowserResult";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import { MapUpdate } from "@/editor/blueprints/MapUpdate";
import { GameMap } from "@/editor/blueprints/GameMap";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { GameInstance } from "@/client/game-instance";
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
  tileset_map: { [tileset_path: string]: Tileset };
  game_map_map: { [map_path: string]: GameMap };
  current_map_index: number;
  modules: { [module_id: string]: Module };
  module_instance_map: { [module_id: string]: string[] };
  selected_resource_tab: number;
  open_resource_paths: string[];
  selected_tileset_path: string;
  selected_tile_id: number;
  selected_tile_position: Point;
  current_main_instance: { instance_id?: string; world_id?: string };
  game_instances: {
    [instance_id: string]: { [world_id: string]: GameInstance };
  };
  side_bar_editor: "module" | "tile" | "map" | "nothing";
  current_file_browser_result: FileBrowserResult;
}
export const use_editor_store = defineStore("editor", {
  state: (): EditorStore => ({
    editor_open: false,
    modules: {},
    module_instance_map: {},
    side_bar_editor: "nothing",
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
    tileset_map: {},
    current_main_instance: {},
    game_instances: {},
    selected_tile_position: { x: 0, y: 0 },
    game_map_map: {},
    conductor: { module_connection_map: {}, resources: [], gid_map: [] },
    current_file_browser_result: { resources: [], dirs: [], dir: "", path: "" },
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
    game_instance_exists(instance_id: string, world_id: string): boolean {
      return (
        !!this.game_instances[instance_id] &&
        !!this.game_instances[instance_id][world_id]
      );
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
    set_game_instances(game_instances: EditorStore["game_instances"]) {
      this.game_instances = { ...game_instances };
    },
    set_selected_tile(tileset_path: string, tile_id: number) {
      this.selected_tileset_path = tileset_path;
      this.selected_tile_id = tile_id;
    },
    set_sidebar_editor(editor: EditorStore["side_bar_editor"]) {
      this.side_bar_editor = editor;
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
    set_current_file_browser_result(result: FileBrowserResult) {
      this.current_file_browser_result = result;
    },
    update_module(module: Partial<Module> & { id: string }) {
      if (module.id) {
        this.modules = {
          ...this.modules,
          [module.id]: { ...this.modules[module.id], ...module },
        };
      }
    },
    delete_module(module_id: string) {
      const modules = {
        ...this.modules,
      };
      delete modules[module_id];
      this.modules = modules;
    },
    create_module(module: Module) {
      this.modules = {
        ...this.modules,
        [module.id]: module,
      };
    },
    set_map(game_map: GameMap) {
      this.game_map_map = {
        ...this.game_map_map,
        [map_key(game_map)]: game_map,
      };
    },
    update_map(
      map_update: Partial<GameMap> & { resource_path: string; name: string },
    ) {
      const key = map_key(map_update);
      this.game_map_map = {
        ...this.game_map_map,
        [key]: { ...this.get_map(key), ...map_update },
      };
    },
    delete_map(game_map: GameMap) {
      const maps = {
        ...this.game_map_map,
      };
      delete maps[map_key(game_map)];
      this.game_map_map = maps;
    },
    get_module(id: string) {
      return this.modules[id];
    },
    get_map(key: string) {
      if (!this.game_map_map[key]) {
        this.get_resource_server(key);
      }
      return this.game_map_map[key];
    },
    get_tileset(tileset_path: string) {
      if (!this.game_map_map[tileset_path]) {
        this.get_resource_server(tileset_path);
      }
      return this.tileset_map[tileset_path];
    },
    set_modules(modules: Module[]) {
      this.modules = modules.reduce(
        (current, module) => ({ ...current, [module.id]: module }),
        {},
      );
    },
    set_tileset(tileset: Tileset) {
      this.tileset_map = {
        ...this.tileset_map,
        [tileset_key(tileset)]: tileset,
      };
    },
    delete_tileset(tileset: Tileset) {
      const tileset_map = { ...this.tileset_map };
      delete tileset_map[tileset_key(tileset)];
      this.tileset_map = tileset_map;
    },
    set_main_door_status(status: boolean) {
      this.main_door_status = status;
    },
    load_modules() {
      send_admin_event("LoadEditorData");
    },
    set_conductor(conductor: Conductor) {
      this.conductor = conductor;
    },
    open_game_instance_server(module_id: string) {
      send_admin_event({ OpenInstance: module_id });
    },
    save_module_server(
      module_id: string,
      module_update: Partial<ModuleUpdate>,
    ) {
      send_admin_event({
        UpdateModule: [
          module_id,
          {
            name: null,
            exit_points: null,
            max_guests: null,
            min_guests: null,
            resources: null,
            insert_points: null,
            ...module_update,
          },
        ],
      });
    },
    toggle_resource_on_module(module_id: string, resource: BlueprintResource) {
      const module = this.get_module(module_id);
      const resource_in_module = module.resources.find(
        (r) => r.path === resource.path,
      );
      if (resource_in_module) {
        this.save_module_server(module.id, {
          resources: module.resources.filter((r) => r.path !== resource.path),
        });
      } else {
        this.save_module_server(module.id, {
          resources: [...module.resources, resource],
        });
      }
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
    browse_folder(path: string) {
      send_admin_event({ BrowseFolder: path });
    },
    get_resource_server(path: string) {
      send_admin_event({ GetResource: path });
    },
    create_map_server(map: GameMap) {
      send_admin_event({ CreateMap: [map.module_id, map] });
    },
    update_map_server(
      map_update: Partial<MapUpdate> &
        Pick<MapUpdate, "resource_path" | "name">,
    ) {
      send_admin_event({
        UpdateMap: {
          chunk: null,
          entities: null,
          ...map_update,
        },
      });
    },
    delete_map_server(map: GameMap) {
      send_admin_event({ DeleteMap: [map.module_id, map] });
    },
    create_tileset_server(tileset: Tileset) {
      send_admin_event({ CreateTileset: tileset });
    },
    update_tileset_server(tileset: Tileset) {
      send_admin_event({ SetTileset: tileset });
    },
    delete_tileset_server(tileset: Tileset) {
      send_admin_event({ DeleteTileset: tileset });
    },
    create_module_server(name: string) {
      send_admin_event({ CreateModule: name });
    },
    delete_module_server(id: string) {
      send_admin_event({ DeleteModule: id });
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
