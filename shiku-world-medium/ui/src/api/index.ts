import { Container, Graphics } from "pixi.js";
import { CommunicationState } from "@/client/communication";
import { Isometry } from "@/client/entities";

export const use_medium_api = (): typeof window.medium =>
  window.medium
    ? window.medium
    : {
        create_collider_graphic: () => [new Graphics(), 1, 2],
        adjust_grid_tile_size: (
          _instance_id: string,
          _world_id: string,
          _size: number,
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
