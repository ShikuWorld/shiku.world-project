import { GuestInput } from "../communication/api/bindings/GuestInput";

export interface GuestInputState {
  key_bindings: { [id: string]: string };
  key_pressed_map: { [id: string]: boolean };
  button_pressed_map: { [id: string]: boolean };
  x_axis_callback: () => number;
  y_axis_callback: () => number;
  is_button_pressed: (button: Button) => boolean;
  is_dirty: boolean;
}

export enum Button {
  Jump = "Jump",
  Left = "Left",
  Right = "Right",
  Up = "Up",
  Down = "Down",
  Start = "Start",
  Action1 = "Action1",
  Action2 = "Action2",
  Exit = "Exit",
}

export function create_guest_input_event(
  guest_input_state: GuestInputState
): GuestInput {
  return {
    x_axis: guest_input_state.x_axis_callback(),
    y_axis: guest_input_state.y_axis_callback(),
    start: guest_input_state.button_pressed_map[Button.Start]
      ? guest_input_state.button_pressed_map[Button.Start]
      : false,
    jump: guest_input_state.button_pressed_map[Button.Jump]
      ? guest_input_state.button_pressed_map[Button.Jump]
      : false,
    up: guest_input_state.button_pressed_map[Button.Up]
      ? guest_input_state.button_pressed_map[Button.Up]
      : false,
    down: guest_input_state.button_pressed_map[Button.Down]
      ? guest_input_state.button_pressed_map[Button.Down]
      : false,
    left: guest_input_state.button_pressed_map[Button.Left]
      ? guest_input_state.button_pressed_map[Button.Left]
      : false,
    right: guest_input_state.button_pressed_map[Button.Right]
      ? guest_input_state.button_pressed_map[Button.Right]
      : false,
    action_1: guest_input_state.button_pressed_map[Button.Action1]
      ? guest_input_state.button_pressed_map[Button.Action1]
      : false,
    action_2: guest_input_state.button_pressed_map[Button.Action2]
      ? guest_input_state.button_pressed_map[Button.Action2]
      : false,
  };
}
