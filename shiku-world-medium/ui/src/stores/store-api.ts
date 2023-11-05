import { use_ui_store } from "@/editor/stores/ui";
import { use_config_store } from "@/editor/stores/config";
import { use_current_module_store } from "@/editor/stores/current-module";
import { use_toast_store } from "@/editor/stores/toast";
import { use_editor_store } from "@/editor/stores/editor";

export function setup_medium_gui_api() {
  window.medium_gui = {
    ui: use_ui_store(),
    editor: use_editor_store(),
    config: use_config_store(),
    current_module: use_current_module_store(),
    toast: use_toast_store(),
  };
}
