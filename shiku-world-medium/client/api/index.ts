import { login } from "../menu/twitch";
import { CommunicationState } from "../communication";
import { GameInstanceMap } from "@/client/game-instance";
import { RenderSystem } from "@/client/renderer";
import {
  create_collider_graphic,
  create_display_object,
  ResourceManagerMap,
} from "@/client/resources";
import { Container } from "pixi.js";
import { set_blueprint_render } from "@/client/renderer/create_game_renderer";
import { adjust_selected_tile_size, toggle_grid } from "@/client/renderer/grid";
import { LayerKind } from "@/editor/blueprints/LayerKind";

export const setup_medium_api = (
  communication_state: CommunicationState,
  instances: GameInstanceMap,
  resource_manager_map: ResourceManagerMap,
  render_system: RenderSystem,
) => {
  window.medium = {
    twitch_login: (communication_state: CommunicationState) =>
      login(communication_state),
    communication_state: communication_state,
    is_instance_ready: (instance_id: string, world_id: string) => {
      return !!instances[instance_id] && !!instances[instance_id][world_id];
    },
    toggle_grid: (instance_id: string, world_id: string) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        toggle_grid(instances[instance_id][world_id].renderer);
      }
    },
    sync_grid_with_layer_p_scaling: (
      instance_id: string,
      world_id: string,
      layer_kind: LayerKind,
    ) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        const renderer = instances[instance_id][world_id].renderer;
        if (renderer.grid) {
          renderer.grid.p_scaling = {
            x: renderer.layer_map[layer_kind].x_pscaling,
            y: renderer.layer_map[layer_kind].y_pscaling,
          };
        }
      }
    },
    create_display_object,
    create_collider_graphic,
    set_blueprint_renderer: (blueprint_render_data) => {
      set_blueprint_render(render_system, instances, blueprint_render_data);
    },
    adjust_brush_hover: (
      instance_id: string,
      world_id: string,
      brush: number[][],
    ) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        adjust_selected_tile_size(
          instances[instance_id][world_id].renderer,
          brush,
        );
      }
    },
    create_container: () => new Container(),
    get_resource_manager: (module_id) => {
      return resource_manager_map[module_id];
    },
    set_camera_iso: (instance_id, world_id, iso) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        instances[instance_id][world_id].renderer.camera.update_camera_position(
          iso,
        );
      }
    },
    set_camera_zoom: (instance_id, world_id, zoom) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        instances[instance_id][world_id].renderer.camera.set_camera_zoom(zoom);
      }
    },
    get_camera_iso: (instance_id, world_id) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        return instances[instance_id][world_id].renderer.camera.camera_isometry;
      }
      return { x: 0, y: 0, rotation: 0 };
    },
    get_camera_zoom: (instance_id, world_id) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        return instances[instance_id][world_id].renderer.camera.zoom;
      }
      return 0;
    },
    swap_main_render_instance: (instance_id, world_id) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        if (
          render_system.current_main_instance.instance_id &&
          render_system.current_main_instance.world_id
        ) {
          if (
            instances[render_system.current_main_instance.instance_id] &&
            instances[render_system.current_main_instance.instance_id][
              render_system.current_main_instance.world_id
            ]
          ) {
            render_system.stage.removeChild(
              instances[render_system.current_main_instance.instance_id][
                render_system.current_main_instance.world_id
              ].renderer.main_container_wrapper,
            );
          }
        }
        render_system.stage.addChild(
          instances[instance_id][world_id].renderer.main_container_wrapper,
        );
        render_system.current_main_instance = { instance_id, world_id };
      }
    },
  };
};
