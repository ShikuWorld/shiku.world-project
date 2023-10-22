import { ResourceManager } from "./index";
import { Config } from "../config";

export function create_resource_manager(): ResourceManager {
  //window.medium_gui.config.set_resource_base_url(Config.get_resource_url());
  return new ResourceManager(`${Config.get_resource_url()}/resources`);
}
