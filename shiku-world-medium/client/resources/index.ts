import {
  AnimatedSprite,
  Assets,
  BaseTexture,
  Graphics as PixijsGraphics,
  Rectangle,
  RenderTexture,
  Sprite,
  Texture,
} from "pixi.js-legacy";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { FrameObject } from "@pixi/sprite-animated";
import { RenderSystem } from "@/client/renderer";
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
  dummy_texture_tileset_missing: RenderTexture;
  dummy_texture_loading: RenderTexture;
  resource_bundle_complete = new SimpleEventDispatcher<{
    module_id: string;
    instance_id: string;
    bundle_name: string;
  }>();
  resources_unload = new SimpleEventDispatcher<void>();

  constructor(
    private _base_url: string,
    renderer: RenderSystem,
  ) {
    const obj = new PixijsGraphics();
    obj.beginFill(0xff00ff);
    obj.drawRect(0, 0, 100, 100);
    this.dummy_texture_tileset_missing = renderer.renderer.generateTexture(obj);
    const obj2 = new PixijsGraphics();
    obj2.beginFill(0xffff00);
    obj2.drawRect(0, 0, 100, 100);
    this.dummy_texture_loading = renderer.renderer.generateTexture(obj2);
  }

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
          for (const res of resource_bundle.assets.filter(
            (a) => a.kind === "Image",
          )) {
            this.image_texture_map[res.path].baseTexture.setResource(
              (r as Texture).baseTexture.resource,
            );
          }
          this._update_uv_maps();
        });
      })
      .with({ LoadTilesets: P.select() }, (tilesets) => {
        this.set_tileset_map(tilesets);
      })
      .with({ UpdateGidMap: P.select() }, (gid_map) => {
        this.gid_map = gid_map;
      })
      .with("UnLoadResources", () => console.log("unload"))
      .exhaustive();
  }

  set_tileset_map(tilesets: Tileset[]) {
    for (const tileset of tilesets) {
      this.tile_set_map[
        `${tileset.resource_path}/${tileset.name}.tileset.json`
      ] = tileset;
    }
  }

  get_sprite_from_graphics(graphics: Graphics): Sprite {
    let sprite: Sprite;

    if (graphics.frame_objects.length > 0) {
      const animated_sprite = new AnimatedSprite(graphics.frame_objects);

      animated_sprite.play();
      sprite = animated_sprite;
    } else {
      sprite = new Sprite(graphics.textures[0]);
    }
    sprite.anchor.set(0, 1);
    return sprite;
  }

  get_graphics_data_by_gid(gid: number): Graphics {
    if (!this.graphic_id_map[gid]) {
      const [tileset, start_gid] = this._get_tileset_by_gid(gid);

      const id_in_tileset = gid - start_gid;

      this.graphic_id_map[gid] = this._calculate_graphics(
        id_in_tileset,
        tileset,
      );
    }

    return this.graphic_id_map[gid];
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
    for (const asset of resource_bundle.assets) {
      match(asset.kind)
        .with("Image", () => {
          this.image_texture_map[asset.path] =
            this.dummy_texture_loading.clone();
        })
        .with("Unknown", () => {})
        .exhaustive();
    }
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
            this.image_texture_map[path].baseTexture.setResource(
              (loaded_resource as Texture).baseTexture.resource,
            );
          })
          .with("Unknown", () => {})
          .exhaustive();
      }
      this._update_uv_maps();
      if (dispatch_resource_bundle_complete) {
        this.resource_bundle_complete.dispatch({
          module_id,
          instance_id,
          bundle_name: resource_bundle.name,
        });
      }
    });
  }

  private _update_uv_maps() {
    for (const g of Object.values(this.graphic_id_map)) {
      for (const t of g.textures) {
        t.updateUvs();
      }
    }
  }

  unload_resources() {}

  private _get_tileset_by_gid(gid: number): [Tileset | undefined, number] {
    for (let i = 0; i < this.gid_map.length; i++) {
      const [path, start_gid] = this.gid_map[i];
      if (start_gid <= gid) {
        return [this.tile_set_map[path], start_gid];
      }
    }
    return [undefined, 0];
  }

  private _calculate_graphics(
    id: number,
    tileset: Tileset | undefined,
  ): Graphics {
    const graphics: Graphics = { textures: [], frame_objects: [] };
    if (id < 0 || !tileset) {
      graphics.textures.push(this.dummy_texture_tileset_missing);
      return graphics;
    }

    ///const animation_frames = tileset.tile_animation_map[id];

    if (tileset.image) {
      const base_texture: BaseTexture =
        this.image_texture_map[tileset.image.path]?.baseTexture;
      if (!base_texture) {
        graphics.textures.push(this.dummy_texture_loading);
        console.error(
          "No base_texture even though there should be a dummy at the very least!",
        );
        return graphics;
      }
      const x = ((id - 1) % tileset.columns) * tileset.tile_width;
      const y = Math.floor(id / tileset.columns) * tileset.tile_height;
      const texture = new Texture(
        base_texture,
        new Rectangle(x, y, tileset.tile_width, tileset.tile_height),
      );
      graphics.textures.push(texture);
    } else {
      graphics.textures.push(this.dummy_texture_loading);
    }

    return graphics;
  }
}

/*
      for (const frame of animation_frames) {
        const x = (frame.tile_id % tileset.columns) * tileset.width;
        const y = Math.floor(frame.tile_id / tileset.columns) * tileset.height;
        const texture = new Texture(
          tileset.tile_set_image_resource.texture.baseTexture,
          new Rectangle(x, y, tileset.width, tileset.height),
        );

        graphics.textures.push(texture);
        graphics.frame_objects.push({ texture, time: frame.duration });
      } */
