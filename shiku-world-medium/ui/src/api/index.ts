import { Container, Graphics } from "pixi.js";
import { CommunicationState } from "@/client/communication";
import { Isometry } from "@/client/entities";
import { LayerKind } from "@/editor/blueprints/LayerKind";

export const use_medium_api = (): typeof window.medium =>
  window.medium
    ? window.medium
    : {
        create_collider_graphic: () => [new Graphics(), 1, 2],
        hide_loading_indicator: () => {},
        reconnect: () => Promise.resolve(),
        is_instance_ready: (_instance_id: string, _world_id: string) => false,
        toggle_grid: (_instance_id: string, _world_id: string) => {},
        adjust_brush_hover: (
          _instance_id: string,
          _world_id: string,
          _brush: number[][],
        ) => {},
        sync_grid_with_layer_p_scaling: (
          _instance_id: string,
          _world_id: string,
          _layer_kind: LayerKind,
        ) => {},
        create_container: () => new Container(),
        set_blueprint_renderer: () => {},
        create_display_object: () => new Container(),
        get_resource_manager: (_: string) => undefined,
        swap_main_render_instance: (_a: string, _b: string) => {},
        communication_state: {} as CommunicationState,
        get_camera_iso: (_a: string, _b: string) => ({
          x: 0,
          y: 0,
          rotation: 0,
        }),
        get_camera_zoom: (_a: string, _b: string) => 0,
        set_camera_iso: (_a: string, _b: string, _c: Isometry) => {},
        set_camera_zoom: (_a: string, _b: string, _c: number) => {},
        twitch_login: (_: CommunicationState) => Promise.resolve(),
      };
