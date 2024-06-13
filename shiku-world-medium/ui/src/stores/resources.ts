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
import { match, P } from "ts-pattern";
import { v4 as uuidv4 } from "uuid";
import { Scene } from "@/editor/blueprints/Scene";
import { FileBrowserResult } from "@/editor/blueprints/FileBrowserResult";
import { GameNode } from "@/editor/blueprints/GameNode";
import { Node2DKind } from "@/editor/blueprints/Node2DKind";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { EntityUpdate } from "@/editor/blueprints/EntityUpdate";
import { SceneNodeUpdate } from "@/client/communication/api/bindings/SceneNodeUpdate";
import { reactive, toRefs } from "vue";
import {
  RenderGraphData,
  use_game_instances_store,
} from "@/editor/stores/game-instances";
import { TilesetUpdate } from "@/client/communication/api/bindings/TilesetUpdate";
import { Script } from "@/editor/blueprints/Script";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { cantor_pair } from "@/client/terrain";

export type Point = { y: number; x: number };

export interface ResourcesStore {
  tileset_map: { [tileset_path: string]: Tileset };
  game_map_map: { [map_path: string]: GameMap };
  conductor: Conductor;
  modules: { [module_id: string]: Module };
  scene_map: { [scene_path: string]: Scene };
  script_map: { [scene_path: string]: Script };
  current_file_browser_result: FileBrowserResult;
}

export const use_resources_store = defineStore("resources", () => {
  const state: ResourcesStore = reactive({
    modules: {},
    conductor: { module_connection_map: {}, resources: [], gid_map: [] },
    tileset_map: {},
    game_map_map: {},
    scene_map: {},
    script_map: {},
    current_file_browser_result: {
      resources: [],
      dirs: [],
      dir: "",
      path: "",
    },
  });

  const actions = {
    update_module(module: Partial<Module> & { id: string }) {
      if (module.id) {
        state.modules = {
          ...state.modules,
          [module.id]: { ...state.modules[module.id], ...module },
        };
      }
    },
    get_or_load_script(
      script_map: { [script_path: string]: Script },
      path: string,
    ): Script | undefined {
      if (!script_map[path]) {
        this.get_resource_server(path);
        return undefined;
      }

      return script_map[path];
    },
    get_or_load_scene(
      scene_map: { [scene_path: string]: Scene },
      path: string,
    ): Scene | undefined {
      if (!scene_map[path]) {
        this.get_resource_server(path);
        return undefined;
      }

      return scene_map[path];
    },
    get_or_load_map(
      map_map: { [map_path: string]: GameMap },
      path: string,
    ): GameMap | undefined {
      if (!map_map[path]) {
        this.get_resource_server(path);
        return undefined;
      }

      return map_map[path];
    },
    get_or_load_tileset(
      tileset_map: { [resource_path: string]: Tileset },
      path: string,
    ): Tileset | undefined {
      if (!tileset_map[path]) {
        this.get_resource_server(path);
        return undefined;
      }

      return tileset_map[path];
    },
    delete_module(module_id: string) {
      const modules = {
        ...state.modules,
      };
      delete modules[module_id];
      state.modules = modules;
    },
    create_module(module: Module) {
      state.modules = {
        ...state.modules,
        [module.id]: module,
      };
    },
    set_map(game_map: GameMap) {
      state.game_map_map = {
        ...state.game_map_map,
        [map_key(game_map)]: game_map,
      };
    },
    update_map(map_update: MapUpdate) {
      const key = map_key(map_update);
      const game_map = state.game_map_map[key];
      if (game_map) {
        const update: Partial<GameMap> = {
          name: map_update.name,
          resource_path: map_update.resource_path,
          main_scene: map_update.scene ?? undefined,
        };
        (Object.keys(update) as Array<keyof GameMap>).forEach(
          (key) => update[key] === undefined && delete update[key],
        );

        state.game_map_map = {
          ...state.game_map_map,
          [key]: {
            ...game_map,
            ...update,
          },
        };
        if (map_update.chunk) {
          const [layer_kind, chunk] = map_update.chunk;
          state.game_map_map[key].terrain[layer_kind][
            cantor_pair(chunk.position[0], chunk.position[1])
          ] = chunk;
        }
        if (map_update.layer_parallax) {
          const [layer_kind, [x, y]] = map_update.layer_parallax;
          state.game_map_map[key].layer_parallax[layer_kind] = [x, y];
        }
      }
    },
    delete_map(game_map: GameMap) {
      const maps = {
        ...state.game_map_map,
      };
      delete maps[map_key(game_map)];
      state.game_map_map = maps;
    },
    set_script(script: Script) {
      state.script_map = {
        ...state.script_map,
        [script_key(script)]: script,
      };
    },
    delete_script(script: Script) {
      const scripts = {
        ...state.script_map,
      };
      delete scripts[script_key(script)];
      state.script_map = scripts;
    },
    get_module(id: string) {
      return state.modules[id];
    },
    set_modules(modules: Module[]) {
      state.modules = modules.reduce(
        (current, module) => ({ ...current, [module.id]: module }),
        {},
      );
    },
    set_tileset(tileset: Tileset) {
      state.tileset_map = {
        ...state.tileset_map,
        [tileset_key(tileset)]: tileset,
      };
    },
    delete_tileset(tileset: Tileset) {
      const tileset_map = { ...state.tileset_map };
      delete tileset_map[tileset_key(tileset)];
      state.tileset_map = tileset_map;
    },
    set_scene(scene: Scene) {
      state.scene_map = {
        ...state.scene_map,
        [scene_key(scene)]: scene,
      };
    },
    update_scene(scene_update: SceneNodeUpdate) {
      match(scene_update)
        .with(
          { UpdateData: P.select() },
          ([resource_path, _, node_id, update]) => {
            const { blueprint_render } = toRefs(use_game_instances_store());
            const { apply_entity_update } = use_game_instances_store();
            if (
              blueprint_render &&
              blueprint_render.value &&
              blueprint_render.value.scene_resource_path === resource_path &&
              blueprint_render.value.render_graph_data
            ) {
              const resource_manager = window.medium.get_resource_manager(
                blueprint_render.value.module_id,
              );
              if (!resource_manager) {
                return;
              }
              apply_entity_update(
                blueprint_render.value.render_graph_data as RenderGraphData,
                {
                  id: node_id,
                  kind: update,
                },
                resource_manager,
              );
            }
          },
        )
        .with(
          { AddChild: P.select() },
          ([resource_path, _, parent_node_id, game_node]) => {
            const { add_child_to_render_graph } = use_game_instances_store();
            const { blueprint_render } = toRefs(use_game_instances_store());
            if (
              blueprint_render &&
              blueprint_render.value &&
              blueprint_render.value.scene_resource_path === resource_path &&
              blueprint_render.value.render_graph_data
            ) {
              const resource_manager = window.medium.get_resource_manager(
                blueprint_render.value.module_id,
              );
              if (!resource_manager) {
                return;
              }
              add_child_to_render_graph(
                blueprint_render.value.render_graph_data as RenderGraphData,
                parent_node_id,
                game_node,
                resource_manager,
              );
            }
          },
        )
        .with({ RemoveChild: P.select() }, ([resource_path, _, game_node]) => {
          const { blueprint_render } = toRefs(use_game_instances_store());
          const { remove_child_from_render_graph, render_key_from_game_node } =
            use_game_instances_store();
          if (
            blueprint_render &&
            blueprint_render.value &&
            blueprint_render.value.scene_resource_path === resource_path &&
            blueprint_render.value.render_graph_data
          ) {
            const resource_manager = window.medium.get_resource_manager(
              blueprint_render.value.module_id,
            );
            if (!resource_manager) {
              return;
            }
            remove_child_from_render_graph(
              blueprint_render.value.render_graph_data as RenderGraphData,
              render_key_from_game_node(game_node),
            );
          }
        })
        .exhaustive();
    },
    delete_scene(scene: Scene) {
      const scene_map = { ...state.scene_map };
      delete scene_map[scene_key(scene)];
      state.scene_map = scene_map;
    },
    load_editor_data() {
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
            main_map: null,
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
    save_conductor_server(conductor: Conductor) {
      send_admin_event({ UpdateConductor: conductor });
    },
    set_conductor(conductor: Conductor) {
      state.conductor = conductor;
    },
    browse_folder(path: string) {
      send_admin_event({ BrowseFolder: path });
    },
    set_current_file_browser_result(result: FileBrowserResult) {
      state.current_file_browser_result = result;
    },
    get_resource_server(path: string) {
      send_admin_event({ GetResource: path });
    },
    create_map_server(map: GameMap) {
      send_admin_event({ CreateMap: [map.module_id, map] });
    },
    create_scene_server(module_id: string, scene: Scene) {
      send_admin_event({ CreateScene: [module_id, scene] });
    },
    create_script_server(module_id: string, script: Script) {
      send_admin_event({ CreateScript: [module_id, script] });
    },
    update_script_server(script: Script) {
      send_admin_event({ UpdateScript: script });
    },
    update_instance_node(
      module_id: string,
      game_instance_id: string,
      world_id: string,
      entity_update: EntityUpdate,
    ) {
      send_admin_event({
        UpdateInstancedNode: [
          module_id,
          game_instance_id,
          world_id,
          entity_update,
        ],
      });
    },
    update_data_in_scene_node_on_server(
      resource_path: string,
      path: number[],
      game_node_id: string,
      entity_update: EntityUpdateKind,
    ) {
      send_admin_event({
        UpdateSceneNode: {
          UpdateData: [resource_path, path, game_node_id, entity_update],
        },
      });
    },
    update_scene_root_with_node(resource_path: string, node: GameNodeKind) {
      send_admin_event({
        OverwriteSceneRoot: [resource_path, node],
      });
    },
    add_child_to_scene_on_server(
      resource_path: string,
      path: number[],
      game_node_id: string,
      data: GameNodeKind,
    ) {
      send_admin_event({
        UpdateSceneNode: {
          AddChild: [resource_path, path, game_node_id, data],
        },
      });
    },
    remove_child_from_scene_on_server(
      resource_path: string,
      path: number[],
      data: GameNodeKind,
    ) {
      send_admin_event({
        UpdateSceneNode: {
          RemoveChild: [resource_path, path, data],
        },
      });
    },
    delete_scene_server(scene: Scene) {
      send_admin_event({ DeleteScene: scene });
    },
    update_map_server(
      map_update: Partial<Omit<MapUpdate, "layer_parallax">> & {
        layer_parallax: [LayerKind, [number, number]] | null;
      } & Pick<MapUpdate, "resource_path" | "name" | "scene">,
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
    create_tileset_server(module_id: string, tileset: Tileset) {
      send_admin_event({ CreateTileset: [module_id, tileset] });
    },
    update_tileset_server(resource_path: string, tile_update: TilesetUpdate) {
      send_admin_event({ UpdateTileset: [resource_path, tile_update] });
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
  };

  return {
    ...toRefs(state),
    ...actions,
  };
});

function send_admin_event(event: AdminToSystemEvent) {
  if (window?.medium?.communication_state?.is_connection_open) {
    window.medium.communication_state.ws_connection.send(JSON.stringify(event));
  } else {
    console.trace("Could not send", event);
  }
}

export function tileset_key(tileset: Tileset) {
  return `${tileset.resource_path}/${tileset.name}.tileset.json`;
}

export function scene_key(scene: Scene) {
  return `${scene.resource_path}/${scene.name}.scene.json`;
}

export function map_key(game_map: { resource_path: string; name: string }) {
  return `${game_map.resource_path}/${game_map.name}.map.json`;
}

export function script_key(script: Script) {
  return `${script.resource_path}/${script.name}.script.json`;
}

export function get_node_by_path(
  node: GameNodeKind,
  path: number[],
): GameNodeKind {
  const n = get_generic_game_node(node);
  if (path.length === 0) {
    return node;
  }
  if (path.length === 1) {
    return n.children[path[0]];
  }
  const p = path.splice(1);
  return get_node_by_path(n.children[path[0]], p);
}

export function get_generic_game_node(node: GameNodeKind): GameNode<unknown> {
  return Object.values(node)[0] as GameNode<unknown>;
}

export function get_game_node_type(
  node: GameNodeKind,
): KeysOfUnion<GameNodeKind> {
  return Object.keys(node)[0] as KeysOfUnion<GameNodeKind>;
}

export function children_of(node: GameNodeKind): Array<GameNodeKind> {
  return get_generic_game_node(node).children;
}

type Node2DKindKeys = KeysOfUnion<Node2DKind>;
export type Node2DTypeKeys = `Node2D-${Node2DKindKeys}`;
export type GameNodeTypeKeys = KeysOfUnion<GameNodeKind> | Node2DTypeKeys;

export function create_game_node(
  game_node_type: GameNodeTypeKeys,
): GameNodeKind | null {
  if (game_node_type.startsWith("Node2D-")) {
    return {
      Node2D: {
        name: game_node_type,
        id: uuidv4(),
        entity_id: null,
        instance_resource_path: null,
        data: {
          transform: {
            position: [0, 0],
            scale: [0, 0],
            velocity: [0, 0],
            rotation: 0,
          },
          kind: create_2d_game_node(game_node_type as Node2DTypeKeys),
        },
        script: null,
        children: [],
      },
    };
  }

  return null;
}

export function create_2d_game_node(
  game_node_type: Node2DTypeKeys,
): Node2DKind {
  return match(game_node_type)
    .with("Node2D-Node2D", (): Node2DKind => ({ Node2D: 0 }))
    .with(
      "Node2D-Instance",
      (): Node2DKind => ({
        Instance: "",
      }),
    )
    .with(
      "Node2D-RigidBody",
      (): Node2DKind => ({
        RigidBody: {
          kinematic_character_controller_props: null,
          body: "Dynamic",
        },
      }),
    )
    .with(
      "Node2D-Collider",
      (): Node2DKind => ({
        Collider: { kind: "Solid", shape: { Ball: 0.5 } },
      }),
    )
    .with(
      "Node2D-Render",
      (): Node2DKind => ({
        Render: { offset: [0, 0], layer: "BG00", kind: { Sprite: 0 } },
      }),
    )
    .exhaustive();
}
