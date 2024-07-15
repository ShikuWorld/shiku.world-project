import { defineStore } from "pinia";
import { ComponentConfig } from "@/editor/ui";

export type CurrentMenuContext = {
  [key: string]:
    | string
    | number
    | null
    | { [key: string]: string | number | null };
};

export interface UiStore {
  menu_open: boolean;
  current_menu?: ComponentConfig;
  current_menu_context?: CurrentMenuContext;
}

export const use_ui_store = defineStore("ui", {
  state: (): UiStore => ({
    menu_open: false,
    current_menu_context: undefined,
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
    set_menu_context(menu_context: {
      [key: string]:
        | string
        | number
        | null
        | { [key: string]: string | number | null };
    }) {
      console.log("Setting menu context", menu_context);
      this.current_menu_context = menu_context;
    },
  },
});
