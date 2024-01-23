"use strict";
(() => {
  // plugins/controller-input.ts
  var plugin_id = "CONTROLLER";
  var controller_input_state;
  function controller_input(guest_input, activate_plugin) {
    controller_input_state = {
      guest_input,
      activate_plugin$: activate_plugin,
      connected_gamepads: {}
    };
    window.addEventListener("gamepadconnected", function(e) {
      const gamepadEvent = e;
      controller_input_state.connected_gamepads[gamepadEvent.gamepad.id] = gamepadEvent.gamepad.index;
    });
    window.addEventListener("gamepaddisconnected", function(e) {
      const gamepadEvent = e;
      delete controller_input_state.connected_gamepads[gamepadEvent.gamepad.id];
    });
  }
  function update_controller_input(is_active) {
    if (!is_active) {
      return;
    }
    for (const key in controller_input_state.connected_gamepads) {
      const gamepad = navigator.getGamepads()[controller_input_state.connected_gamepads[key]];
      if (!gamepad) {
        continue;
      }
      if (gamepad.buttons[9].pressed) {
        setButton(controller_input_state.guest_input, "Start" /* Start */, true);
      } else {
        setButton(controller_input_state.guest_input, "Start" /* Start */, false);
      }
      if (gamepad.buttons[1].pressed || gamepad.buttons[1].value > 0) {
        setButton(controller_input_state.guest_input, "Right" /* Right */, true);
        setButton(controller_input_state.guest_input, "Left" /* Left */, true);
        setButton(controller_input_state.guest_input, "Jump" /* Jump */, false);
        continue;
      }
      if (gamepad.axes[0] > 0.5) {
        setButton(controller_input_state.guest_input, "Right" /* Right */, true);
        setButton(controller_input_state.guest_input, "Left" /* Left */, false);
      }
      if (gamepad.axes[0] >= -0.5 && gamepad.axes[0] <= 0.5) {
        setButton(controller_input_state.guest_input, "Right" /* Right */, false);
        setButton(controller_input_state.guest_input, "Left" /* Left */, false);
      }
      if (gamepad.axes[0] < -0.5) {
        setButton(controller_input_state.guest_input, "Left" /* Left */, true);
        setButton(controller_input_state.guest_input, "Right" /* Right */, false);
      }
      if (gamepad.axes[1] > 0.5) {
        setButton(controller_input_state.guest_input, "Up" /* Up */, false);
        setButton(controller_input_state.guest_input, "Down" /* Down */, true);
      }
      if (gamepad.axes[1] >= -0.5 && gamepad.axes[1] <= 0.5) {
        setButton(controller_input_state.guest_input, "Up" /* Up */, false);
        setButton(controller_input_state.guest_input, "Down" /* Down */, false);
      }
      if (gamepad.axes[1] < -0.5) {
        setButton(controller_input_state.guest_input, "Up" /* Up */, true);
        setButton(controller_input_state.guest_input, "Down" /* Down */, false);
      }
      if (gamepad.buttons[0].pressed || gamepad.buttons[0].value > 0) {
        setButton(controller_input_state.guest_input, "Jump" /* Jump */, true);
      } else {
        setButton(controller_input_state.guest_input, "Jump" /* Jump */, false);
      }
      if (gamepad.buttons[1].pressed || gamepad.buttons[1].value > 0) {
        setButton(controller_input_state.guest_input, "Action1" /* Action1 */, true);
      } else {
        setButton(controller_input_state.guest_input, "Action1" /* Action1 */, false);
      }
      if (gamepad.buttons[2].pressed || gamepad.buttons[2].value > 0) {
        setButton(controller_input_state.guest_input, "Action2" /* Action2 */, true);
      } else {
        setButton(controller_input_state.guest_input, "Action2" /* Action2 */, false);
      }
      if (gamepad.buttons[2].pressed || gamepad.buttons[2].value > 0) {
        setButton(controller_input_state.guest_input, "Action2" /* Action2 */, true);
      } else {
        setButton(controller_input_state.guest_input, "Action2" /* Action2 */, false);
      }
    }
  }
  if (window.register_input_plugin) {
    window.register_input_plugin({
      id: plugin_id,
      plugin_options: {},
      initialize: controller_input,
      update: update_controller_input
    });
  }
  function setButton(guest_input, button, button_state) {
    if (guest_input.button_pressed_map[button] !== button_state) {
      guest_input.button_pressed_map[button] = button_state;
      guest_input.is_dirty = true;
    }
  }
})();
//# sourceMappingURL=controller-input.js.map
