import { InputPlugin } from "@/client/plugins";
import { login } from "@/client/menu/twitch";
import { CommunicationState } from "@/client/communication";
import { use_ui_store } from "@/gui/stores/ui";
import { use_config_store } from "@/gui/stores/config";
import { use_current_module_store } from "@/gui/stores/current-module";
import { use_toast_store } from "@/gui/stores/toast";

declare global {
  interface Window {
    register_input_plugin: (plugin: InputPlugin) => void;
    medium: {
      twitch_login: typeof login;
      communication_state: CommunicationState;
    };
    medium_gui: {
      ui: ReturnType<typeof use_ui_store>;
      config: ReturnType<typeof use_config_store>;
      toast: ReturnType<typeof use_toast_store>;
      current_module: ReturnType<typeof use_current_module_store>;
    };
  }
}
