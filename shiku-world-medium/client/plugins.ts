import { GuestInputState } from "./input";
import { SimpleEventDispatcher } from "strongly-typed-events";

export type InputPlugin<T = object> = {
  id: string;
  initialize: (
    guest_input: GuestInputState,
    on_input: SimpleEventDispatcher<string>,
  ) => void;
  plugin_options: T;
  update?: (is_active: boolean) => void;
};

const input_plugins: InputPlugin[] = [];
const activate_plugin: SimpleEventDispatcher<string> =
  new SimpleEventDispatcher<string>();
let active_plugin: string;

activate_plugin.subscribe((plugin_id) => {
  active_plugin = plugin_id;
});

export function get_plugin(id: string) {
  return input_plugins.find((i) => i.id === id);
}

function set_active_plugin(name: string, input_toggle_icon: Element) {
  switch (name) {
    case "CONTROLLER":
      input_toggle_icon.textContent = "videogame_asset";
      break;
    case "KEYBOARD":
      input_toggle_icon.textContent = "keyboard";
      break;
    case "MOUSE":
      input_toggle_icon.textContent = "mouse";
      break;
    default:
      return;
  }

  try {
    localStorage.setItem("preferred_input", name);
  } catch (e) {
    console.error(
      "Seems like you block local storage or something, you'll have to choose your input method on every reload.",
    );
  }

  activate_plugin.dispatch(name);
}

export function setup_plugin_system() {
  window.register_input_plugin = (plugin) => {
    input_plugins.push(plugin);
  };

  document.addEventListener("DOMContentLoaded", () => {
    const input_toggle = document.getElementById("toggle-input-method");
    const input_toggle_icon = document.querySelector(
      "#toggle-input-method span",
    );
    if (!input_toggle || !input_toggle_icon) {
      console.error("No input toggle?!");
      return;
    }

    input_toggle.addEventListener("click", () => {
      if (active_plugin === "MOUSE") {
        set_active_plugin("CONTROLLER", input_toggle_icon);
        return;
      }

      if (active_plugin === "CONTROLLER") {
        if (input_plugins.find((i) => i.id === "KEYBOARD")) {
          set_active_plugin("KEYBOARD", input_toggle_icon);
        } else {
          set_active_plugin("MOUSE", input_toggle_icon);
        }
        return;
      }

      if (active_plugin === "KEYBOARD") {
        set_active_plugin("MOUSE", input_toggle_icon);
        return;
      }
    });
  });
}

export const initialize_input_plugins = (guest_input: GuestInputState) => {
  for (const plugin of input_plugins) {
    plugin.initialize(guest_input, activate_plugin);
    plugin.update = plugin.update ? plugin.update : () => {};
  }

  const input_toggle_icon = document.querySelector("#toggle-input-method span");
  if (!input_toggle_icon) {
    console.error("No input toggle?!");
    return;
  }
  try {
    const preferred_input = localStorage.getItem("preferred_input");
    if (preferred_input) {
      set_active_plugin(preferred_input, input_toggle_icon);
    } else {
      set_active_plugin("MOUSE", input_toggle_icon);
    }
  } catch (e) {
    set_active_plugin("MOUSE", input_toggle_icon);
    console.error(
      "Seems like you block local storage or something, you'll have to choose your input method on every reload.",
    );
  }
};

export const update_input_plugins = () => {
  for (const plugin of input_plugins) {
    plugin.update && plugin.update(plugin.id === active_plugin);
  }
};
