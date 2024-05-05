import {
  AnimatedSprite,
  Assets,
  TextureSource,
  Rectangle,
  Sprite,
  Texture,
  FrameObject,
  Container,
  Text,
  Graphics as PixijsGraphics,
} from "pixi.js";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { RenderSystem } from "@/client/renderer";
import { ResourceEvent } from "@/client/communication/api/bindings/ResourceEvent";
import { match, P } from "ts-pattern";
import { ResourceBundle } from "@/client/communication/api/bindings/ResourceBundle";
import { Tileset } from "@/client/communication/api/blueprints/Tileset";
import { LoadResource } from "@/client/communication/api/bindings/LoadResource";
import { GidMap } from "@/editor/blueprints/GidMap";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { RENDER_SCALE } from "@/shared/index";

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
  dummy_texture_tileset_missing: Texture;
  dummy_texture_loading: Texture;
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
    this.dummy_texture_tileset_missing = renderer.dummy_texture_tileset_missing;
    this.dummy_texture_loading = renderer.dummy_texture_loading;
  }

  set_tileset_map(tilesets: Tileset[]) {
    for (const tileset of tilesets) {
      this.tile_set_map[
        `${tileset.resource_path}/${tileset.name}.tileset.json`
      ] = tileset;
    }
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
          this.image_texture_map[asset.path] = Texture.from(
            this.dummy_texture_loading.source,
          );
          this.image_texture_map[asset.path].source.update();
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
      for (const load_resource of resource_bundle.assets) {
        const path = load_resource.path;
        const loaded_resource = r[path];
        if (!loaded_resource) {
          console.error(`${path} did not load?!`);
          continue;
        }
        match(path_to_resource_map[path].kind)
          .with("Image", () => {
            this.image_texture_map[path].source.resource = (
              loaded_resource as Texture
            ).source.resource;
            this.image_texture_map[path].source.update();
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

  private _update_uv_maps() {
    for (const g of Object.values(this.graphic_id_map)) {
      for (const t of g.textures) {
        t.updateUvs();
      }
    }
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
            this.image_texture_map[res.path].source.resource = (
              r[res.path] as Texture
            ).source.resource;
            this.image_texture_map[res.path].source.update();
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

  unload_resources() {}

  get_sprite_from_graphics(graphics: Graphics): Sprite {
    let sprite: Sprite;

    if (graphics.frame_objects.length > 0) {
      const animated_sprite = new AnimatedSprite(graphics.frame_objects);

      animated_sprite.play();
      sprite = animated_sprite;
    } else {
      sprite = Sprite.from(graphics.textures[0]);
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
      const texture_source: TextureSource =
        this.image_texture_map[tileset.image.path]?.source;
      if (!texture_source) {
        graphics.textures.push(this.dummy_texture_loading);
        console.error(
          "No base_texture even though there should be a dummy at the very least!",
        );
        return graphics;
      }
      const x = ((id - 1) % tileset.columns) * tileset.tile_width;
      const y = Math.floor(id / tileset.columns) * tileset.tile_height;
      const texture = new Texture({
        source: texture_source,
        frame: new Rectangle(x, y, tileset.tile_width, tileset.tile_height),
      });
      graphics.textures.push(texture);
    } else {
      graphics.textures.push(this.dummy_texture_loading);
    }

    return graphics;
  }
}

export function create_display_object(
  node: GameNodeKind,
  resource_manager: ResourceManager,
): Container {
  const container = new Container();
  match(node)
    .with({ Instance: P.select() }, () => {
      console.error("No instances can be displayed!");
    })
    .with({ Node2D: P.select() }, (game_node) => {
      container.x = game_node.data.transform.position[0] * RENDER_SCALE;
      container.y = game_node.data.transform.position[1] * RENDER_SCALE;
      container.rotation = game_node.data.transform.rotation;
      match(game_node.data.kind)
        .with({ Node2D: P.select() }, () => {
          //container.addChild(new Text(game_node.name, { fill: "white" }));
        })
        .with({ Render: P.select() }, (render) => {
          const display_object = match(render.kind)
            .with({ Sprite: P.select() }, (gid) => {
              const graphics = resource_manager.get_graphics_data_by_gid(gid);
              return resource_manager.get_sprite_from_graphics(graphics);
            })
            .with(
              { AnimatedSprite: P.select() },
              (gid) =>
                new Text(`Animated Sprite not implemented. gid: ${gid}`, {
                  fill: "red",
                }),
            )
            .exhaustive();
          container.addChild(display_object);
        })
        .with({ RigidBody: P.select() }, (rigid_body) => {
          console.log("rb", rigid_body);
        })
        .with({ Collider: P.select() }, (collider) => {
          match(collider.shape)
            .with({ Ball: P.select() }, (radius) => {
              const graphics = new PixijsGraphics()
                .circle(0, 0, radius * RENDER_SCALE)
                .stroke({
                  color: "#ff0000",
                  width: 1,
                });
              container.addChild(graphics);
            })
            .with({ CapsuleX: P.select() }, ([_half_y, _radius]) => {})
            .with({ CapsuleY: P.select() }, ([_half_x, _radius]) => {})
            .with({ Cuboid: P.select() }, ([_a, _b]) => {})
            .exhaustive();
        })
        .exhaustive();
    })
    .exhaustive();
  return container;
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
