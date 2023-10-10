import { use_ui_store } from "./ui";
import { use_config_store } from "@/stores/config";
import { use_current_module_store } from "@/stores/current-module";
import { use_toast_store } from "@/stores/toast";

export function setup_medium_gui_api() {
  window.medium_gui = {
    ui: use_ui_store(),
    config: use_config_store(),
    current_module: use_current_module_store(),
    toast: use_toast_store(),
  };
}
