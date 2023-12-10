import { ResourceManager } from "./index";
import { Config } from "../config";
import { RenderSystem } from "@/client/renderer";

export function create_resource_manager(
  render_system: RenderSystem,
): ResourceManager {
  //window.medium_gui.config.set_resource_base_url(Config.get_resource_url());
  return new ResourceManager(`${Config.get_resource_url()}`, render_system);
}
