import { Assets, Texture } from "pixi.js-legacy";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { FrameObject } from "@pixi/sprite-animated";
import { InstanceRendering } from "@/client/renderer";
import { ResourceEvent } from "@/client/communication/api/bindings/ResourceEvent";
import { match, P } from "ts-pattern";
import { ResourceBundle } from "@/client/communication/api/bindings/ResourceBundle";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { LoadResource } from "@/client/communication/api/bindings/LoadResource";
import { GidMap } from "@/editor/blueprints/GidMap";

export interface Graphics {
  textures: Texture[];
  frame_objects: FrameObject[];
}

export type ResourceManagerMap = { [module_id: string]: ResourceManager };

export class ResourceManager {
  image_texture_map: {
    [path: string]: Texture;
  } = {};
  graphic_id_map: {
    [gid: string]: Graphics;
  } = {};
  gid_map: GidMap = [];
  tilesets: Tileset[] = [];
  tile_set_map: {
    [path: string]: Tileset;
  } = {};

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
        Assets.loadBundle(resource_bundle.name).then((r) => {
          console.log(r);
          /*for (const res of resource_bundle.assets) {
            this.resource_map[res.path] = r;
          }*/
        });
      })
      .with({ LoadTilesets: P.select() }, (tilesets) => {
        for (const tileset of tilesets) {
          this.tile_set_map[tileset.resource_path] = tileset;
        }
      })
      .with({ UpdateGidMap: P.select() }, (gid_map) => {
        console.log(gid_map);
      })
      .with("UnLoadResources", () => console.log("unload"))
      .exhaustive();
  }

  get_graphics_data_by_gid(
    gid: number,
    _renderer: InstanceRendering,
  ): Graphics {
    if (!this.graphic_id_map[gid]) {
      /*const tile_set: TileSet = ResourceManager._get_tileset_by_gid(
        gid,
        this.resourceModule.tile_sets,
      );

      const id_in_tileset = gid - tile_set.start_gid;

      this.resourceModule.graphic_id_map[gid] =
        ResourceManager._calculate_graphics(id_in_tileset, tile_set, renderer);*/
    }

    return {
      frame_objects: [],
      textures: [],
    };
  }

  load_resource_bundle(
    module_id: string,
    instance_id: string,
    resource_bundle: ResourceBundle,
    dispatch_resource_bundle_complete: boolean = false,
  ) {
    const bundle_id = `${module_id}-${resource_bundle.name}`;
    const path_to_resource_map = resource_bundle.assets.reduce(
      (acc, r) => ({ ...acc, [r.path]: r }),
      {} as { [path: string]: LoadResource },
    );
    Assets.addBundle(
      bundle_id,
      resource_bundle.assets.map((asset) => ({
        alias: asset.path,
        src: `${this._base_url}/${asset.path}?q=${asset.cache_hash}`,
      })),
    );
    Assets.loadBundle(bundle_id).then((r) => {
      const loaded: { [path: string]: Texture | "other" } = r;
      for (const [path, loaded_resource] of Object.entries(loaded)) {
        match(path_to_resource_map[path].kind)
          .with("Image", () => {
            this.image_texture_map[path] = loaded_resource as Texture;
          })
          .with("Unknown", () => {})
          .exhaustive();
      }
      if (dispatch_resource_bundle_complete) {
        this.resource_bundle_complete.dispatch({
          module_id,
          instance_id,
          bundle_name: resource_bundle.name,
        });
      }
    });
  }

  unload_resources() {}
}
