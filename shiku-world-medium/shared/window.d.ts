import { InputPlugin } from "@/client/plugins";
import { login } from "@/client/menu/twitch";
import { CommunicationState } from "@/client/communication";
import { use_ui_store } from "@/ui/stores/ui";
import { use_config_store } from "@/ui/stores/config";
import { use_current_module_store } from "@/ui/stores/current-module";
import { use_toast_store } from "@/ui/stores/toast";
import { use_editor_store } from "@/ui/stores/editor";

declare global {
  interface Window {
    register_input_plugin: (plugin: InputPlugin) => void;
    medium: {
      twitch_login: typeof login;
      communication_state: CommunicationState;
    };
    medium_gui: {
      ui: ReturnType<typeof use_ui_store>;
      editor: ReturnType<typeof use_editor_store>;
      config: ReturnType<typeof use_config_store>;
      toast: ReturnType<typeof use_toast_store>;
      current_module: ReturnType<typeof use_current_module_store>;
    };
  }
}
