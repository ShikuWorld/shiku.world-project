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

declare global {
  interface Window {
    register_input_plugin: (plugin: InputPlugin) => void;
    medium: {
      twitch_login: typeof login;
      communication_state: CommunicationState;
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
      config: ReturnType<typeof use_config_store>;
      toast: ReturnType<typeof use_toast_store>;
      current_module: ReturnType<typeof use_current_module_store>;
    };
  }
}
