import { Button, GuestInputState } from "@/client/input";
import { SimpleEventDispatcher } from "strongly-typed-events";

const plugin_id = "KEYBOARD";
let activate_plugin$: SimpleEventDispatcher<string>;
let is_active = false;

function keyboard_input(
  guest_input: GuestInputState,
  activate_plugin: SimpleEventDispatcher<string>
) {
  register_key_bindings(guest_input);
  activate_plugin$ = activate_plugin;
  activate_plugin$.subscribe((active_plugin) => {
    is_active = active_plugin === plugin_id;
  });
  document.addEventListener("keydown", onKeyDown.bind(guest_input));
  document.addEventListener("keyup", onKeyUp.bind(guest_input));
}

function register_key_bindings(guest_input: GuestInputState) {
  set_keyboard_button_mapping(guest_input, Button.Up, KeyCode.W);
  set_keyboard_button_mapping(guest_input, Button.Right, KeyCode.D);
  set_keyboard_button_mapping(guest_input, Button.Down, KeyCode.S);
  set_keyboard_button_mapping(guest_input, Button.Left, KeyCode.A);
  set_keyboard_button_mapping(guest_input, Button.Jump, KeyCode.SPACE);
  set_keyboard_button_mapping(guest_input, Button.Exit, KeyCode.ESCAPE);
  set_keyboard_button_mapping(guest_input, Button.Action1, KeyCode.E);
  set_keyboard_button_mapping(guest_input, Button.Action2, KeyCode.F);

  //Setting some standard callback functions for x- and y-axis
  guest_input.x_axis_callback = () => {
    if (guest_input.button_pressed_map[Button.Left]) return -1;
    if (guest_input.button_pressed_map[Button.Right]) return 1;
    return 0;
  };
  guest_input.y_axis_callback = () => {
    if (guest_input.button_pressed_map[Button.Up]) return -1;
    if (guest_input.button_pressed_map[Button.Down]) return 1;
    return 0;
  };
}

enum KeyCode {
  W = "w",
  A = "a",
  S = "s",
  D = "d",
  E = "e",
  F = "f",
  SPACE = " ",
  ESCAPE = "Escape",
  LEFT_ARROW = 37,
  UP_ARROW = 38,
  RIGHT_ARROW = 39,
  DOWN_ARROW = 40,
}

function set_keyboard_button_mapping(
  guest_input: GuestInputState,
  button: Button,
  key_code: KeyCode
) {
  guest_input.key_bindings[key_code as string] = button as string;
  guest_input.key_pressed_map[key_code as string] = false;
  guest_input.button_pressed_map[button as string] = false;
}

function onKeyDown(e: KeyboardEvent) {
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

function onKeyUp(e: KeyboardEvent) {
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
    plugin_options: {},
  });
}
