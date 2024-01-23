"use strict";
(() => {
  // plugins/keyboard-input.ts
  var plugin_id = "KEYBOARD";
  var activate_plugin$;
  var is_active = false;
  function keyboard_input(guest_input, activate_plugin) {
    register_key_bindings(guest_input);
    activate_plugin$ = activate_plugin;
    activate_plugin$.subscribe((active_plugin) => {
      is_active = active_plugin === plugin_id;
    });
    document.addEventListener("keydown", onKeyDown.bind(guest_input));
    document.addEventListener("keyup", onKeyUp.bind(guest_input));
  }
  function register_key_bindings(guest_input) {
    set_keyboard_button_mapping(guest_input, "Up" /* Up */, "w" /* W */);
    set_keyboard_button_mapping(guest_input, "Right" /* Right */, "d" /* D */);
    set_keyboard_button_mapping(guest_input, "Down" /* Down */, "s" /* S */);
    set_keyboard_button_mapping(guest_input, "Left" /* Left */, "a" /* A */);
    set_keyboard_button_mapping(guest_input, "Jump" /* Jump */, " " /* SPACE */);
    set_keyboard_button_mapping(guest_input, "Exit" /* Exit */, "Escape" /* ESCAPE */);
    set_keyboard_button_mapping(guest_input, "Action1" /* Action1 */, "e" /* E */);
    set_keyboard_button_mapping(guest_input, "Action2" /* Action2 */, "f" /* F */);
    guest_input.x_axis_callback = () => {
      if (guest_input.button_pressed_map["Left" /* Left */])
        return -1;
      if (guest_input.button_pressed_map["Right" /* Right */])
        return 1;
      return 0;
    };
    guest_input.y_axis_callback = () => {
      if (guest_input.button_pressed_map["Up" /* Up */])
        return -1;
      if (guest_input.button_pressed_map["Down" /* Down */])
        return 1;
      return 0;
    };
  }
  function set_keyboard_button_mapping(guest_input, button, key_code) {
    guest_input.key_bindings[key_code] = button;
    guest_input.key_pressed_map[key_code] = false;
    guest_input.button_pressed_map[button] = false;
  }
  function onKeyDown(e) {
    if (!is_active) {
      return;
    }
    if (this.key_bindings[e.key] === void 0 || e.repeat) {
      return;
    }
    this.button_pressed_map[this.key_bindings[e.key]] = true;
    this.key_pressed_map[e.key] = true;
    this.is_dirty = true;
  }
  function onKeyUp(e) {
    if (this.key_bindings[e.key] === void 0) {
      return;
    }
    this.button_pressed_map[this.key_bindings[e.key]] = false;
    this.key_pressed_map[e.key] = false;
    this.is_dirty = true;
  }
  if (window.register_input_plugin) {
    window.register_input_plugin({
      initialize: keyboard_input,
      id: plugin_id,
      plugin_options: {}
    });
  }
})();
//# sourceMappingURL=keyboard-input.js.map
