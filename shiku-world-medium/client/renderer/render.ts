import { Renderer } from "./index";

export function render(renderer: Renderer) {
  renderer.renderer.render(renderer.stage);
}
