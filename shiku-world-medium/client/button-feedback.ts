import { Button, GuestInputState } from "./input";

export const setup_button_feedback = (): ((
  _guest_input: GuestInputState
) => void) => {
  const button_element_map: { [key: string]: HTMLElement } = {};
  const input_feedback = document.getElementById("input-feedback");
  const input_toggle = document.getElementById("toggle-input-feedback");
  const input_toggle_icon = document.querySelector(
    "#toggle-input-feedback span"
  );
  input_toggle.addEventListener("click", () => {
    if (input_toggle_icon.textContent === "visibility") {
      input_toggle_icon.textContent = "visibility_off";
      input_feedback.className = "feedback-disabled";
    } else {
      input_toggle_icon.textContent = "visibility";
      input_feedback.className = "";
    }
  });

  for (const button in Button) {
    button_element_map[button] =
      document.getElementById(`input-feedback-${button}`) ||
      ({ className: "" } as HTMLElement);
  }

  return (guest_input) => {
    for (const button in Button) {
      if (guest_input.button_pressed_map[button]) {
        button_element_map[button].className = "button-active";
      } else {
        button_element_map[button].className = "button-inactive";
      }
    }
  };
};
