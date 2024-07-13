import { defineStore } from "pinia";
import { ComponentConfig } from "@/editor/ui";

export interface UiStore {
  menu_open: boolean;
  current_menu?: ComponentConfig;
}

export const use_ui_store = defineStore("ui", {
  state: (): UiStore => ({
    menu_open: false,
  }),
  actions: {
    open_menu() {
      this.menu_open = true;
    },
    close_menu() {
      this.menu_open = false;
    },
    toggle_menu() {
      this.menu_open = !this.menu_open;
    },
    set_menu(menu: ComponentConfig) {
      console.log("Setting menu", menu);
      this.current_menu = menu;
    },
  },
});
