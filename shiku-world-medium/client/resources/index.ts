import { Texture } from "pixi.js-legacy";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { Resource as ResourceLoadingDefinition } from "../communication/api/bindings/Resource";
import { FrameObject } from "@pixi/sprite-animated";

interface TileSet {
  rawData: Document;
  name: string;
  id_to_image_resource_map?: Map<number, { width: number; height: number }>;
  start_gid: number;
  tile_animation_map: TileAnimationMap;
  width: number;
  height: number;
  columns: number;
}

interface ResourceModule {
  resource_map: {
    [resource_name: string]: {
      definition: ResourceLoadingDefinition;
    };
  };
  graphic_id_map: {
    [gid: string]: Graphics;
  };
  tile_sets: TileSet[];
}

export interface Graphics {
  textures: Texture[];
  frame_objects: FrameObject[];
}

interface TileAnimationMap {
  [tile_id: number]: Array<{ tile_id: number; duration: number }>;
}

export class ResourceManager {
  resourceModuleMap: {
    [module_name: string]: ResourceModule;
  };

  resources_complete: SimpleEventDispatcher<{
    module_name: string;
  }>;
  resources_unload: SimpleEventDispatcher<{
    module_name: string;
  }>;

  constructor(private base_url: string) {
    this.resourceModuleMap = {};
    this.resources_complete = new SimpleEventDispatcher();
    this.resources_unload = new SimpleEventDispatcher();
  }

  get_resource_module(module_name: string): ResourceModule {
    if (!this.resourceModuleMap[module_name]) {
      this.resourceModuleMap[module_name] = {
        resource_map: {},
        graphic_id_map: {},
        tile_sets: [],
      };
    }

    return this.resourceModuleMap[module_name];
  }

  add_resource_to_loading_queue(
    _module_name: string,
    _resource_loading_definition: ResourceLoadingDefinition,
  ) {}

  unload_resources(_module_name: string) {}

  start_loading(_module_name: string) {
    // this.get_resource_module(module_name).loader.load();
  }
}
