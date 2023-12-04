import { Assets, Texture } from "pixi.js-legacy";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { Resource as ResourceLoadingDefinition } from "../communication/api/bindings/Resource";
import { FrameObject } from "@pixi/sprite-animated";
import { InstanceRendering } from "@/client/renderer";
import { ResourceEvent } from "@/client/communication/api/bindings/ResourceEvent";
import { match, P } from "ts-pattern";
import { ResourceBundle } from "@/client/communication/api/bindings/ResourceBundle";

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
  resourceModule: ResourceModule = {
    resource_map: {},
    graphic_id_map: {},
    tile_sets: [],
  };

  resource_bundle_complete = new SimpleEventDispatcher<{
    module_id: string;
    instance_id: string;
    bundle_name: string;
  }>();
  resources_unload = new SimpleEventDispatcher<void>();

  constructor(private _base_url: string) {}

  handle_resource_event(resource_event: ResourceEvent) {
    match(resource_event)
      .with({ LoadResource: P.select() }, (resource_bundle) => {
        Assets.addBundle(
          resource_bundle.name,
          resource_bundle.assets.map((asset) => ({
            alias: asset.path,
            src: `${this._base_url}/${asset.path}?q=${asset.cache_hash}`,
          })),
        );
        Assets.loadBundle(resource_bundle.name).then(() => {
          for (const res of resource_bundle.assets) {
            console.log("Do something with res", res);
          }
        });
      })
      .with("UnLoadResource", () => console.log("unload"))
      .exhaustive();
  }

  get_graphics_data_by_gid(
    _gid: number,
    _renderer: InstanceRendering,
  ): Graphics {
    /*if (!this.resourceModule.graphic_id_map[gid]) {
      const tile_set: TileSet = ResourceManager._get_tileset_by_gid(
        gid,
          this.resourceModule.tile_sets,
      );

      const id_in_tileset = gid - tile_set.start_gid;

      this.resourceModule.graphic_id_map[gid] = ResourceManager._calculate_graphics(
        id_in_tileset,
        tile_set,
        renderer,
      );
    }*/

    return {
      frame_objects: [],
      textures: [],
    };
  }

  load_resource_bundle(
    module_id: string,
    instance_id: string,
    resource_bundle: ResourceBundle,
  ) {
    const bundle_id = `${module_id}-${resource_bundle.name}`;
    Assets.addBundle(
      bundle_id,
      resource_bundle.assets.map((asset) => ({
        alias: asset.path,
        src: `${this._base_url}/${asset.path}?q=${asset.cache_hash}`,
      })),
    );
    Assets.loadBundle(bundle_id).then(() => {
      for (const res of resource_bundle.assets) {
        console.log("Do something with res", res);
      }
      this.resource_bundle_complete.dispatch({
        module_id,
        instance_id,
        bundle_name: resource_bundle.name,
      });
    });
  }

  unload_resources() {}
}
