import { Button, GuestInputState } from "@/client/input";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { InputPlugin } from "@/client/plugins";
import {match} from "ts-pattern";
import {MouseInputSchema} from "@/client/communication/api/bindings/MouseInputSchema";

const plugin_id = "MOUSE";
let activate_plugin$: SimpleEventDispatcher<string>;
let mouse_active = false;

export type MousePluginType = InputPlugin<{mouse_mode: MouseInputSchema}>;

const plugin: MousePluginType = { initialize: mouse_input, id: plugin_id, plugin_options: {mouse_mode: 'PurelyDirectionalNoJump'} };

function mouse_input(
  guest_input: GuestInputState,
  activate_plugin: SimpleEventDispatcher<string>
) {
  activate_plugin$ = activate_plugin;

  activate_plugin$.subscribe((input_plugin_id) => {
    if (mouse_active && input_plugin_id !== plugin_id) {
      set_button(guest_input, Button.Left, false);
      set_button(guest_input, Button.Right, false);
      set_button(guest_input, Button.Up, false);
      set_button(guest_input, Button.Down, false);
      set_button(guest_input, Button.Jump, false);
    }

    mouse_active = input_plugin_id === plugin_id;
  });

  document.addEventListener("mousemove", (mouse_event: MouseEvent) => {
    if (mouse_active) {
      update_mouse_input_button(mouse_event, guest_input);
    }
  });

  document.addEventListener(
    "visibilitychange",
    function () {
      if (document.hidden) {
        set_button(guest_input, Button.Left, false);
        set_button(guest_input, Button.Right, false);
        set_button(guest_input, Button.Up, false);
        set_button(guest_input, Button.Down, false);
        set_button(guest_input, Button.Jump, false);
      }
    },
    false
  );

  document.addEventListener("mouseleave", function (event) {
    if (
      event.clientY <= 0 ||
      event.clientX <= 0 ||
      event.clientX >= window.innerWidth ||
      event.clientY >= window.innerHeight
    ) {
      set_button(guest_input, Button.Left, false);
      set_button(guest_input, Button.Right, false);
      set_button(guest_input, Button.Up, false);
      set_button(guest_input, Button.Down, false);
      set_button(guest_input, Button.Jump, false);
    }
  });
}

function update_mouse_input_button(
  mouse_event: MouseEvent,
  guest_input: GuestInputState
) {
  const canvas = document.getElementById("canvas");
  const width = canvas.offsetWidth / 2;
  const height = canvas.offsetHeight / 2;

  const cursor_x = mouse_event.x - width;
  const cursor_y = mouse_event.y - height;

  match(plugin.plugin_options.mouse_mode)
    .with('UpIsJumpAndNoDown', () => {
      set_button(guest_input, Button.Jump, cursor_y < -50);
      left_right_standard(cursor_x, guest_input);
    })
    .with('PurelyDirectionalNoJump', () => {
      up_down_standard(cursor_y, guest_input);
      left_right_standard(cursor_x, guest_input);
    })
    .exhaustive();
}

function left_right_standard(cursor_x: number, guest_input: GuestInputState) {
  if (cursor_x < -50) {
    set_button(guest_input, Button.Left, true);
    set_button(guest_input, Button.Right, false);
  }

  if (cursor_x >= -50 && cursor_x <= 50) {
    set_button(guest_input, Button.Left, false);
    set_button(guest_input, Button.Right, false);
  }

  if (cursor_x > 50) {
    set_button(guest_input, Button.Right, true);
    set_button(guest_input, Button.Left, false);
  }
}

function up_down_standard(cursor_y: number, guest_input: GuestInputState) {
  if (cursor_y < -50) {
    set_button(guest_input, Button.Up, true);
    set_button(guest_input, Button.Down, false);
  }

  if (cursor_y >= -50 && cursor_y <= 50) {
    set_button(guest_input, Button.Up, false);
    set_button(guest_input, Button.Down, false);
  }

  if (cursor_y > 50) {
    set_button(guest_input, Button.Up, false);
    set_button(guest_input, Button.Down, true);
  }
}

function set_button(
  guest_input: GuestInputState,
  button: Button,
  button_state: boolean
) {
  if (guest_input.button_pressed_map[button] !== button_state) {
    guest_input.button_pressed_map[button] = button_state;
    guest_input.is_dirty = true;
  }
}

if (window.register_input_plugin) {
  window.register_input_plugin(plugin);
}
