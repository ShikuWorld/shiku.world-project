import { GuestInputState } from "./index";

export function create_guest_input(): GuestInputState {
  const guest_input: GuestInputState = {
    key_bindings: {},
    button_pressed_map: {},
    key_pressed_map: {},
    x_axis_callback: () => 0,
    y_axis_callback: () => 0,
    is_button_pressed: () => true,
    is_dirty: true,
  };

  window.addEventListener("blur", () => {
    for (const key of Object.keys(guest_input.key_bindings)) {
      guest_input.button_pressed_map[guest_input.key_bindings[key]] = false;
      guest_input.key_pressed_map[key] = false;
    }
    guest_input.is_dirty = true;
  });
  window.addEventListener("contextmenu", () => {
    for (const key of Object.keys(guest_input.key_bindings)) {
      guest_input.button_pressed_map[guest_input.key_bindings[key]] = false;
      guest_input.key_pressed_map[key] = false;
    }
    guest_input.is_dirty = true;
  });

  return guest_input;
}
