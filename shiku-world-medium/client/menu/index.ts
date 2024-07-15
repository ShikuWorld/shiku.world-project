import { ResourceManager } from "../resources";
import { ComponentConfig } from "../../ui/src/ui";
import { loginMenuConfig } from "@/client/login-menu";
import { reconnectMenuConfig } from "@/client/reconnect-menu";

export class MenuSystem {
  private _menus: { [name: string]: ComponentConfig };
  static static_menus = {
    LoginMenu: "login-menu",
    ReconnectMenu: "reconnect-menu",
  };

  constructor() {
    this._menus = {};
    this.create_menu_from_config(
      loginMenuConfig,
      MenuSystem.static_menus.LoginMenu,
    );
    this.create_menu_from_config(
      reconnectMenuConfig,
      MenuSystem.static_menus.ReconnectMenu,
    );
  }

  create_menu_from_config(config: ComponentConfig, menu_name: string) {
    if (!this._menus[menu_name]) {
      this._menus[menu_name] = config;
    } else {
      throw Error("Menu already existed");
    }
  }

  get(menu_name: string): ComponentConfig {
    const menu = this._menus[menu_name];
    if (!menu) {
      throw Error(`Tried to get menu that did not exist. ${menu_name}`);
    }
    return menu;
  }

  activate(
    menu_name: string,
    context?: {
      [key: string]:
        | string
        | number
        | null
        | { [key: string]: string | number | null };
    },
  ) {
    const menu = this._menus[menu_name];
    if (!menu) {
      throw Error(`Tried to activate menu that did not exist. ${menu_name}`);
    }
    if (context) {
      window.medium_gui.ui.set_menu_context(context);
    }
    window.medium_gui.ui.set_menu(menu);
    window.medium_gui.ui.open_menu();
  }

  toggle(menu_name: string) {
    const menu = this._menus[menu_name];
    if (!menu) {
      throw Error("Tried to deactivate menu that did not exist.");
    }
    window.medium_gui.ui.set_menu(menu);
    window.medium_gui.ui.toggle_menu();
  }

  deactivate(menu_name: string) {
    const menu = this._menus[menu_name];
    if (!menu) {
      throw Error("Tried to deactivate menu that did not exist.");
    }
    window.medium_gui.ui.close_menu();
  }

  remove(menu_name: string) {
    const menu = this._menus[menu_name];
    if (!menu) {
      throw Error("Tried to deactivate menu that did not exist.");
    }
    window.medium_gui.ui.close_menu();
    delete this._menus[menu_name];
  }
}

export function setup_automatic_menu_creation(
  _resource_manager: ResourceManager,
  _menu_system: MenuSystem,
) {
  /*resource_manager.resources_complete.sub((event) => {
    for (const resource of Object.values(event.resources)) {
      if (resource.xhrType == "json") {
        const menu_config = resource.data as ComponentConfig;
        menu_system.create_menu_from_config(menu_config, resource.name);
      }
    }
  });

  resource_manager.resources_unload.sub((event) => {
    for (const resource of Object.values(event.loader.resources)) {
      if (resource.xhrType == "json") {
        menu_system.remove(resource.name);
      }
    }
  });*/
}
