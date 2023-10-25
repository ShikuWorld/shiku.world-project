import { RenderSystem } from "./index";

export function render(render_system: RenderSystem) {
  render_system.renderer.render(render_system.stage);
}
