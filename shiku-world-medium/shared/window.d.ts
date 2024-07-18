import { InputPlugin } from "@/client/plugins";
import { login } from "@/client/menu/twitch";
import { CommunicationState } from "@/client/communication";
import { use_ui_store } from "@/editor/stores/ui";
import { use_config_store } from "@/editor/stores/config";
import { use_current_module_store } from "@/editor/stores/current-module";
import { use_toast_store } from "@/editor/stores/toast";
import { use_editor_store } from "@/editor/stores/editor";
import { Isometry } from "@/client/entities";
import { use_resources_store } from "@/editor/stores/resources";
import {
  GameInstancesStore,
  use_game_instances_store,
} from "@/editor/stores/game-instances";
import {
  create_collider_graphic,
  create_display_object,
  ResourceManager,
} from "@/client/resources";
import { Container } from "pixi.js";
import { LayerKind } from "@/editor/blueprints/LayerKind";

declare global {
  interface Window {
    register_input_plugin: (plugin: InputPlugin) => void;
    medium: {
      twitch_login: typeof login;
      hide_loading_indicator: () => void;
      communication_state: CommunicationState;
      reconnect: () => Promise<void>;
      is_instance_ready: (instance_id: string, world_id: string) => boolean;
      get_resource_manager: (module_id: string) => ResourceManager | undefined;
      set_blueprint_renderer: (
        blueprint_render_data: GameInstancesStore["blueprint_render"],
      ) => void;
      sync_grid_with_layer_p_scaling: (
        instance_id: string,
        world_id: string,
        layer_kind: LayerKind,
      ) => void;
      toggle_grid: (instance_id: string, world_id: string) => void;
      adjust_brush_hover: (
        instance_id: string,
        world_id: string,
        brush: number[][],
      ) => void;
      create_display_object: typeof create_display_object;
      create_collider_graphic: typeof create_collider_graphic;
      toggle_terrain_collisions: () => void;
      create_container: () => Container;
      set_camera_iso: (
        instance_id: string,
        world_id: string,
        iso: Isometry,
      ) => void;
      set_camera_zoom: (
        instance_id: string,
        world_id: string,
        zoom: number,
      ) => void;
      get_camera_iso: (instance_id: string, world_id: string) => Isometry;
      get_camera_zoom: (instance_id: string, world_id: string) => number;
      swap_main_render_instance: (
        instance_id: string,
        world_id: string,
      ) => void;
    };
    medium_gui: {
      ui: ReturnType<typeof use_ui_store>;
      editor: ReturnType<typeof use_editor_store>;
      resources: ReturnType<typeof use_resources_store>;
      game_instances: ReturnType<typeof use_game_instances_store>;
      config: ReturnType<typeof use_config_store>;
      toast: ReturnType<typeof use_toast_store>;
      current_module: ReturnType<typeof use_current_module_store>;
    };
  }
}
