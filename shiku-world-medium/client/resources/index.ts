import {
  AnimatedSprite,
  Assets,
  Container,
  FrameObject,
  Graphics as PixijsGraphics,
  Rectangle,
  Sprite,
  Texture,
  TextureSource,
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
import { Collider } from "@/editor/blueprints/Collider";
import { create_dummy_pic } from "@/client/renderer/create_game_renderer";
import { CharAnimationToTilesetMap } from "@/editor/blueprints/CharAnimationToTilesetMap";

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
  character_animation_to_tileset_map: CharAnimationToTilesetMap = {};
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
    public module_id: string,
  ) {
    this.dummy_texture_tileset_missing = renderer.dummy_texture_tileset_missing;
    this.dummy_texture_loading = renderer.dummy_texture_loading;
    this.graphic_id_map["0"] = { textures: [Texture.EMPTY], frame_objects: [] };
  }

  set_tileset_map(tilesets: Tileset[]) {
    for (const tileset of tilesets) {
      this.tile_set_map[
        `${tileset.resource_path}/${tileset.name}.tileset.json`
      ] = tileset;
    }
  }

  async add_loading_to_texture_map(path: string) {
    const loading = Texture.from(await create_dummy_pic("#FF00ff"));
    this.image_texture_map[path] = Texture.from(loading.source);
    this.image_texture_map[path].source.update();
  }

  async load_resource_bundle(
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
      await match(asset.kind)
        .with("Image", async () => {
          if (!this.image_texture_map[asset.path]) {
            console.log("Adding", asset);
            await this.add_loading_to_texture_map(asset.path);
            console.log("oh oh", asset);
          }
          return Promise.resolve();
        })
        .with("Unknown", async () => Promise.resolve())
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
      this._update_uv_maps();
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
        console.log("gid_map update", gid_map);
        this.gid_map = gid_map;
      })
      .with("UnLoadResources", () => console.log("unload"))
      .exhaustive();
  }

  get_sprite_from_graphics(graphics: Graphics): Sprite {
    let sprite: Sprite;

    if (graphics.frame_objects.length > 0) {
      const animated_sprite = new AnimatedSprite(graphics.frame_objects);

      animated_sprite.play();
      sprite = animated_sprite;
    } else {
      sprite = Sprite.from(graphics.textures[0]);
    }
    sprite.anchor.set(0.5, 0.5);
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

  get_graphics_by_id_and_tileset_path(
    id_in_tileset: number,
    tileset_path: string,
  ): Graphics {
    const tileset = this.tile_set_map[tileset_path];
    if (!tileset) {
      console.error("No tileset for", tileset_path, this.module_id);
      return {
        textures: [this.dummy_texture_tileset_missing],
        frame_objects: [],
      };
    }
    const start_gid = this.gid_map.find((g) => g[0] === tileset_path)?.[1] || 0;
    const gid = id_in_tileset + start_gid;
    if (!this.graphic_id_map[gid]) {
      this.graphic_id_map[gid] = this._calculate_graphics(
        id_in_tileset,
        tileset,
      );
    }

    return this.graphic_id_map[gid];
  }

  private _get_tileset_by_gid(gid: number): [Tileset | undefined, number] {
    let selected_gid_index = 0;
    for (let i = 0; i < this.gid_map.length; i++) {
      const start_gid = this.gid_map[i][1];
      if (gid < start_gid) {
        break;
      }
      selected_gid_index = i;
    }
    if (!this.gid_map[selected_gid_index]) {
      console.error("No tileset for gid", gid, this.gid_map, this.module_id);
      return [undefined, 0];
    }
    const [path, start_gid] = this.gid_map[selected_gid_index];
    return [this.tile_set_map[path], start_gid];
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
      const texture_source: TextureSource | undefined =
        this.image_texture_map[tileset.image.path]?.source;
      if (!texture_source) {
        console.log("@@@@@@@@@@", Object.keys(this.image_texture_map));
        graphics.textures.push(this.dummy_texture_loading);
        console.error(
          "No base_texture even though there should be a dummy at the very least!",
        );
        return graphics;
      }
      const x = ((id - 1) % tileset.columns) * tileset.tile_width;
      const y = Math.floor((id - 1) / tileset.columns) * tileset.tile_height;
      const texture = new Texture({
        source: texture_source,
        frame: new Rectangle(x, y, tileset.tile_width, tileset.tile_height),
      });
      graphics.textures.push(texture);
    } else {
      const image_path = tileset.tiles[id]?.image?.path;
      if (!image_path) {
        graphics.textures.push(this.dummy_texture_loading);
        console.error("Could not find image path for tile!?");
        return graphics;
      }
      const texture_source: TextureSource | undefined =
        this.image_texture_map[image_path]?.source;
      if (!texture_source) {
        graphics.textures.push(this.dummy_texture_loading);
        console.error("Could not find image source for tile!?");
        return graphics;
      }
      const texture = new Texture({
        source: texture_source,
      });
      graphics.textures.push(texture);
    }

    return graphics;
  }
}

export function create_display_object(
  node: GameNodeKind,
  resource_manager: ResourceManager,
  show_colliders: boolean = false,
): Container {
  const container = new Container();
  match(node)
    .with({ Node2D: P.select() }, (game_node) => {
      container.x = game_node.data.transform.position[0] * RENDER_SCALE;
      container.y = game_node.data.transform.position[1] * RENDER_SCALE;
      container.rotation = game_node.data.transform.rotation;

      match(game_node.data.kind)
        .with({ Node2D: P.select() }, { Instance: P.select() }, () => {
          //container.addChild(new Text(game_node.name, { fill: "white" }));
        })
        .with({ Render: P.select() }, (render) => {
          const display_object = match(render.kind)
            .with({ Sprite: P.select() }, ([tileset_path, id_in_tileset]) => {
              const graphics =
                resource_manager.get_graphics_by_id_and_tileset_path(
                  id_in_tileset,
                  tileset_path,
                );
              return resource_manager.get_sprite_from_graphics(graphics);
            })
            .with(
              { AnimatedSprite: P.select() },
              ([char_anim_resource_path, id_in_tileset]) => {
                const graphics =
                  resource_manager.get_graphics_by_id_and_tileset_path(
                    id_in_tileset,
                    resource_manager.character_animation_to_tileset_map[
                      char_anim_resource_path
                    ],
                  );
                return resource_manager.get_sprite_from_graphics(graphics);
              },
            )
            .exhaustive();
          container.addChild(display_object);
        })
        .with({ RigidBody: P.select() }, (_) => {})
        .with({ Collider: P.select() }, (collider) => {
          const [graphics, pivot_x, pivot_y] =
            create_collider_graphic(collider);
          container.addChild(graphics);
          container.pivot.x = pivot_x * RENDER_SCALE;
          container.pivot.y = pivot_y * RENDER_SCALE;
          container.visible = show_colliders;
        })
        .exhaustive();
    })
    .exhaustive();
  return container;
}

export function create_collider_graphic(
  collider: Collider,
): [PixijsGraphics, number, number] {
  return match(collider.shape)
    .with({ Ball: P.select() }, (radius): [PixijsGraphics, number, number] => {
      const graphics = new PixijsGraphics()
        .circle(0, 0, radius * RENDER_SCALE)
        .stroke({
          color: "#ff0000",
          width: 1,
        });
      return [graphics, 0, 0];
    })
    .with(
      { CapsuleX: P.select() },
      ([_half_y, _radius]): [PixijsGraphics, number, number] => {
        const graphics = new PixijsGraphics()
          .circle(0, 0, RENDER_SCALE)
          .stroke({
            color: "#ff0000",
            width: 1,
          });
        return [graphics, 1, 1];
      },
    )
    .with(
      { CapsuleY: P.select() },
      ([_half_x, _radius]): [PixijsGraphics, number, number] => {
        const graphics = new PixijsGraphics()
          .circle(0, 0, RENDER_SCALE)
          .stroke({
            color: "#ff0000",
            width: 1,
          });
        return [graphics, 1, 1];
      },
    )
    .with(
      { Cuboid: P.select() },
      ([a, b]): [PixijsGraphics, number, number] => {
        const graphics = new PixijsGraphics()
          .rect(
            -a * RENDER_SCALE,
            -b * RENDER_SCALE,
            a * 2 * RENDER_SCALE,
            b * 2 * RENDER_SCALE,
          )
          .stroke({
            color: "#ff0000",
            width: 1,
          });
        return [graphics, 0, 0];
      },
    )
    .exhaustive();
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
