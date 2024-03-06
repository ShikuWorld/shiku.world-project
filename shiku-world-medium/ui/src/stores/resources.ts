import { defineStore } from "pinia";
import type { Module } from "@/editor/blueprints/Module";
import { ModuleUpdate } from "@/editor/blueprints/ModuleUpdate";
import { Conductor } from "@/editor/blueprints/Conductor";
import { BlueprintResource } from "@/editor/blueprints/BlueprintResource";
import { MapUpdate } from "@/editor/blueprints/MapUpdate";
import { GameMap } from "@/editor/blueprints/GameMap";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { KeysOfUnion } from "@/editor/utils";
import { match } from "ts-pattern";
import { v4 as uuidv4 } from "uuid";
import { Scene } from "@/editor/blueprints/Scene";
import { FileBrowserResult } from "@/editor/blueprints/FileBrowserResult";

export type Point = { y: number; x: number };

export interface ResourcesStore {
  tileset_map: { [tileset_path: string]: Tileset };
  game_map_map: { [map_path: string]: GameMap };
  modules: { [module_id: string]: Module };
  scene_map: { [scene_path: string]: Scene };
  current_file_browser_result: FileBrowserResult;
}
export const use_resources_store = defineStore("resources", {
  state: (): ResourcesStore => ({
    modules: {},
    tileset_map: {},
    game_map_map: {},
    scene_map: {},
    current_file_browser_result: { resources: [], dirs: [], dir: "", path: "" },
  }),
  actions: {
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
    set_scene(scene: Scene) {
      this.scene_map = {
        ...this.scene_map,
        [scene_key(scene)]: scene,
      };
    },
    get_scene(resource_path: string) {
      if (!this.scene_map[resource_path]) {
        this.get_resource_server(resource_path);
      }
      return this.scene_map[resource_path];
    },
    delete_scene(scene: Scene) {
      const scene_map = { ...this.scene_map };
      delete scene_map[scene_key(scene)];
      this.scene_map = scene_map;
    },
    load_modules() {
      send_admin_event("LoadEditorData");
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
    browse_folder(path: string) {
      send_admin_event({ BrowseFolder: path });
    },
    set_current_file_browser_result(result: FileBrowserResult) {
      this.current_file_browser_result = result;
    },
    get_resource_server(path: string) {
      send_admin_event({ GetResource: path });
    },
    create_map_server(map: GameMap) {
      send_admin_event({ CreateMap: [map.module_id, map] });
    },
    create_scene_server(scene: Scene) {
      send_admin_event({ CreateScene: scene });
    },
    update_scene_server(scene: Scene) {
      send_admin_event({ UpdateScene: scene });
    },
    delete_scene_server(scene: Scene) {
      send_admin_event({ DeleteScene: scene });
    },
    update_map_server(
      map_update: Partial<MapUpdate> &
        Pick<MapUpdate, "resource_path" | "name" | "scene">,
    ) {
      send_admin_event({
        UpdateMap: {
          chunk: null,
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

export function scene_key(scene: Scene) {
  return `${scene.resource_path}/${scene.name}.scene.json`;
}

export function resource_key(resource: BlueprintResource) {
  return `${resource.dir}/${resource.file_name}`;
}

export function map_key(game_map: { resource_path: string; name: string }) {
  return `${game_map.resource_path}/${game_map.name}.map.json`;
}

export function create_game_node(
  game_node_type: KeysOfUnion<GameNodeKind>,
): GameNodeKind {
  return match(game_node_type)
    .with(
      "RigidBody",
      (): GameNodeKind => ({
        RigidBody: {
          name: "RigidBody",
          id: uuidv4(),
          data: {
            position: [0, 0],
            velocity: [0, 0],
            rotation: 0,
            body: "Dynamic",
          },
          script: null,
          children: [],
        },
      }),
    )
    .with(
      "Collider",
      (): GameNodeKind => ({
        Collider: {
          name: "Collider",
          id: uuidv4(),
          data: { kind: "Solid", shape: { Ball: 0 } },
          script: null,
          children: [],
        },
      }),
    )
    .with(
      "Node",
      (): GameNodeKind => ({
        Node: {
          name: "Node",
          id: uuidv4(),
          data: "",
          script: null,
          children: [],
        },
      }),
    )
    .with(
      "Render",
      (): GameNodeKind => ({
        Render: {
          name: "Render",
          id: uuidv4(),
          data: { offset: [0, 0], layer: "BG00", kind: "Sprite" },
          script: null,
          children: [],
        },
      }),
    )
    .with(
      "Instance",
      (): GameNodeKind => ({
        Instance: {
          name: "Render",
          id: uuidv4(),
          data: "",
          script: null,
          children: [],
        },
      }),
    )
    .exhaustive();
}
