import { InstanceRendering } from "@/client/renderer";
import {
  create_entity_manager,
  create_render_graph,
  EntityManager,
} from "@/client/entities";
import { create_terrain_manager, TerrainManager } from "@/client/terrain";
import { create_instance_rendering } from "@/client/renderer/create_game_renderer";
import { GameSystemToGuestEvent } from "@/client/communication/api/bindings/GameSystemToGuestEvent";
import { match, P } from "ts-pattern";
import { MediumDataStorage } from "@/client/communication/api/bindings/MediumDataStorage";
import { get_plugin } from "@/client/plugins";
import { MousePluginType } from "../plugins/mouse-input";
import { MenuSystem } from "@/client/menu";
import { ResourceManager } from "@/client/resources";
import { TerrainParams } from "@/editor/blueprints/TerrainParams";
import { set_container_to_viewport_coordinate } from "@/client/camera";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { update_grid } from "@/client/renderer/grid";
import { RenderGraph } from "@/client/render-graph";

export type GameInstanceMap = {
  [instance_id: string]: { [world_id: string]: GameInstance };
};

export class GameInstance {
  renderer: InstanceRendering;
  entity_manager: EntityManager;
  render_graph: RenderGraph;
  terrain_manager: TerrainManager;
  layer_map_keys: LayerKind[];

  constructor(
    public id: string,
    public module_name: string,
    public world_id: string,
    terrain_params: TerrainParams,
  ) {
    this.render_graph = create_render_graph();
    this.renderer = create_instance_rendering(terrain_params);
    this.entity_manager = create_entity_manager();
    this.terrain_manager = create_terrain_manager(terrain_params);
    this.layer_map_keys = Object.keys(this.renderer.layer_map) as LayerKind[];
  }

  update() {
    this.renderer.camera.update_camera_position_from_ref(this.entity_manager);

    for (const layerName of this.layer_map_keys) {
      const parallax_container = this.renderer.layer_map[layerName];
      set_container_to_viewport_coordinate(
        this.renderer.camera.camera_isometry,
        parallax_container,
      );
    }
    update_grid(this.renderer.camera.camera_isometry, this.renderer);
  }

  handle_position_update(
    position_update: Array<[string, number, number, number]>,
  ) {
    for (const [entity_id, x, y, r] of position_update) {
      this.entity_manager.update_entity_position(entity_id, x, y, r);
    }
  }

  handle_game_system_event(
    game_system_event: GameSystemToGuestEvent,
    menu_system: MenuSystem,
    resource_manager: ResourceManager,
  ) {
    match(game_system_event)
      .with({ SetCamera: P.select() }, ([entity_id, camera_settings]) => {
        this.renderer.camera.set_camera_ref(entity_id, this.module_name);
        this.renderer.camera.set_camera_settings(camera_settings);
      })
      .with({ OpenMenu: P.select() }, (menuName) => {
        menu_system.activate(menuName);
      })
      .with({ CloseMenu: P.select() }, (menuName) => {
        menu_system.deactivate(menuName);
      })
      .with({ ShowScene: P.select() }, (scene) => {
        this.render_graph.render_graph_from_scene(scene, resource_manager);
        this.renderer.layer_map.FG10.addChild(
          this.render_graph.render_root.container,
        );
      })
      .with({ UpdateEntity: P.select() }, (node) => {
        this.render_graph.apply_node_update(node, resource_manager);
      })
      .with({ RemoveSceneNodes: P.select() }, (node_ids) => {
        console.log(node_ids);
      })
      .with({ ChangeEntity: P.select() }, ([update_entities, _moduleName]) => {
        console.log(update_entities);
      })
      .with({ ShowTerrain: P.select() }, (layers) => {
        for (const [layer_kind, chunks] of layers) {
          for (const chunk of chunks) {
            this.terrain_manager.add_chunk(
              resource_manager,
              this.renderer,
              layer_kind,
              chunk,
            );
          }
        }
      })
      .with({ UpdateDataStore: P.select() }, (store_update) => {
        try {
          const update = JSON.parse(store_update) as MediumDataStorage;
          window.medium_gui.current_module.set_data(update);
        } catch (e) {
          console.error("Could not parse store update!");
        }
      })
      .with({ SetMouseInputSchema: P.select() }, (mouse_mode) => {
        const mouse_plugin = get_plugin("MOUSE") as MousePluginType | undefined;
        if (mouse_plugin) {
          mouse_plugin.plugin_options.mouse_mode = mouse_mode;
        }
      })
      /*.with({ ShowEffects: P.select() }, (show_effects) => {
                                                                    for (const show_effect of show_effects) {
                                                                      match(show_effect)
                                                                        .with({ SimpleImageEffect: P.select() }, (simple_image_effect) => {
                                                                          this.entity_manager.add_simple_image_effect(
                                                                            simple_image_effect,
                                                                            this.renderer,
                                                                            resource_manager,
                                                                          );
                                                                        })
                                                                        .with({ ShakeScreenEffect: P.select() }, (shake_screen_effect) => {
                                                                          new_shaker({
                                                                            target: this.renderer.main_container,
                                                                            isBidirectional: shake_screen_effect.is_bidirectional,
                                                                            shakeCountMax: shake_screen_effect.shake_count_max,
                                                                            shakeDelay: shake_screen_effect.shake_delay,
                                                                            shakeAmount: shake_screen_effect.shake_amount,
                                                                          }).shake();
                                                                        })
                                                                        .exhaustive();
                                                                    }
                                                                  })*/
      .with({ SetParallax: P.select() }, (_layer_parallax) => {
        /*for (const key in this.renderer.mainContainer) {
                                                                                          const parallax_container =
                                                                                            this.renderer.layerContainer[key as LayerName];
                                                                                          parallax_container.y_pscaling = 1.0;
                                                                                          parallax_container.x_pscaling = 1.0;
                                                                                        }
                                                                                        for (const [layer_name, parallax] of layer_parallax) {
                                                                                          if (layer_name === "Menu") {
                                                                                            continue;
                                                                                          }
                                                                                          this.renderer.layerContainer[layer_name].x_pscaling = parallax[0];
                                                                                          this.renderer.layerContainer[layer_name].y_pscaling = parallax[1];
                                                                                        }*/
      })
      .exhaustive();
  }

  destroy() {}
}

export function create_new_game_instance(
  id: string,
  module_name: string,
  world_id: string,
  terrain_params: TerrainParams,
) {
  return new GameInstance(id, module_name, world_id, terrain_params);
}
