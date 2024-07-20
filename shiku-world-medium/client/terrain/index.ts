import { AnimatedSprite, Container, Ticker } from "pixi.js";
import { ResourceManager } from "../resources";
import { InstanceRendering } from "../renderer";
import { TerrainParams } from "@/editor/blueprints/TerrainParams";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { Chunk } from "@/editor/blueprints/Chunk";
import {
  create_basic_fade_in_animation,
  create_basic_fade_out_animation,
  SpriteEffect,
  SpriteEffectProperties,
} from "@/client/sprite-animations";
import { update } from "@tweenjs/tween.js";

export function to_natural(num: number): number {
  if (num < 0) {
    return -2 * num - 1;
  } else {
    return 2 * num;
  }
}

export function cantor_pair(x: number, y: number): number {
  const xx = to_natural(x);
  const yy = to_natural(y);
  return ((xx + yy) * (xx + yy + 1)) / 2 + yy;
}

export function create_terrain_manager(
  terrain_params: TerrainParams,
): TerrainManager {
  return new TerrainManager(terrain_params);
}

type TileEffect = {
  base_props: SpriteEffectProperties;
  fade_in: SpriteEffect;
  fade_out: SpriteEffect;
  sprite: AnimatedSprite;
  gid: number;
};

type TileEffectMap = {
  [key: string]: TileEffect;
};

export class TerrainManager {
  private _chunk_map: Map<
    LayerKind,
    Map<number, { container: Container; data: Chunk }>
  > = new Map();

  private _active_animations: TileEffect[] = [];
  sprite_by_gid_map: {
    [gid: string]: {
      effects: Set<string>;
      main_animation_sprite_key: string | null;
    };
  } = {};
  tile_effects_map: TileEffectMap = {};

  constructor(public terrain_params: TerrainParams) {
    Ticker.shared.add(() => {
      this.sync_sprite_animations();
    });
  }

  remove_all_chunks_for_module(renderer: InstanceRendering) {
    for (const [layer, layer_chunks] of this._chunk_map.entries()) {
      for (const chunk of Object.values(layer_chunks)) {
        renderer.layer_map[layer].removeChild(chunk.container);
      }
    }
  }

  update_animations_for_animated_sprites(
    resource_manager: ResourceManager,
    gid: number,
  ) {
    if (!this.sprite_by_gid_map[gid]) {
      return;
    }
    const effects_for_gid = this.sprite_by_gid_map[gid];
    const graphics = resource_manager.get_graphics_data_by_gid(gid);
    const is_animated = graphics.frame_objects.length > 0;
    for (const tile_key of effects_for_gid.effects.values()) {
      this.tile_effects_map[tile_key].sprite.textures =
        graphics.frame_objects.length > 0
          ? graphics.frame_objects
          : graphics.textures;
    }
    if (is_animated) {
      if (effects_for_gid.main_animation_sprite_key == null) {
        effects_for_gid.main_animation_sprite_key = effects_for_gid.effects
          .values()
          .next().value as string;
      }
      this.tile_effects_map[
        effects_for_gid.main_animation_sprite_key
      ].sprite.play();
    }

    if (!is_animated && effects_for_gid.main_animation_sprite_key != null) {
      effects_for_gid.main_animation_sprite_key = null;
    }
  }

  sync_sprite_animations() {
    for (const sprite_map of Object.values(this.sprite_by_gid_map)) {
      if (sprite_map.main_animation_sprite_key === null) {
        continue;
      }
      let main_sprite =
        this.tile_effects_map[sprite_map.main_animation_sprite_key]?.sprite;
      // give role of main sprite to a different sprite if main sprite was removed
      if (!main_sprite) {
        sprite_map.main_animation_sprite_key =
          (sprite_map.effects.values().next().value as string) ?? null;
        main_sprite =
          this.tile_effects_map[sprite_map.main_animation_sprite_key]?.sprite;
        if (!main_sprite) {
          continue;
        }
        main_sprite.play();
      }
      const current_frame = main_sprite.currentFrame;
      for (const tile_key of sprite_map.effects.values()) {
        if (tile_key === sprite_map.main_animation_sprite_key) {
          continue;
        }
        this.tile_effects_map[tile_key].sprite.currentFrame = current_frame;
      }
    }
  }

  update_effects() {
    this._active_animations = this._active_animations.filter((tile_effect) => {
      update(window.performance.now());
      tile_effect.sprite.position.x =
        tile_effect.base_props.pos_x +
        tile_effect.fade_in.add_props.pos_x +
        tile_effect.fade_out.add_props.pos_x;
      tile_effect.sprite.position.y =
        tile_effect.base_props.pos_y +
        tile_effect.fade_in.add_props.pos_y +
        tile_effect.fade_out.add_props.pos_y;
      tile_effect.sprite.rotation =
        tile_effect.base_props.rotation +
        tile_effect.fade_in.add_props.rotation +
        tile_effect.fade_out.add_props.rotation;
      tile_effect.sprite.alpha =
        tile_effect.base_props.alpha +
        tile_effect.fade_in.add_props.alpha +
        tile_effect.fade_out.add_props.alpha;
      tile_effect.sprite.scale.x =
        tile_effect.base_props.scale_x +
        tile_effect.fade_in.add_props.scale_x +
        tile_effect.fade_out.add_props.scale_x;
      tile_effect.sprite.scale.y =
        tile_effect.base_props.scale_y +
        tile_effect.fade_in.add_props.scale_y +
        tile_effect.fade_out.add_props.scale_y;
      return (
        tile_effect.fade_in.all_tweens.some((t) => t.isPlaying()) ||
        tile_effect.fade_out.all_tweens.some((t) => t.isPlaying())
      );
    });
  }

  add_chunk(
    resource_manager: ResourceManager,
    renderer: InstanceRendering,
    layer_kind: LayerKind,
    chunk: Chunk,
  ) {
    if (!this._chunk_map.has(layer_kind)) {
      this._chunk_map.set(layer_kind, new Map());
    }
    const chunk_map = this._chunk_map.get(layer_kind)!;
    const chunk_key = cantor_pair(chunk.position[0], chunk.position[1]);
    if (!chunk_map.has(chunk_key)) {
      const chunk_map_entry = {
        container: new Container(),
        data: chunk,
        animations: {},
      };
      chunk_map_entry.container.x =
        chunk.position[0] *
        this.terrain_params.chunk_size *
        this.terrain_params.tile_width;
      chunk_map_entry.container.y =
        chunk.position[1] *
        this.terrain_params.chunk_size *
        this.terrain_params.tile_height;
      chunk_map.set(chunk_key, chunk_map_entry);
      renderer.layer_map[layer_kind].addChild(chunk_map_entry.container);
    }

    const chunk_map_entry = chunk_map.get(chunk_key)!;
    chunk_map_entry.data = chunk;
    /*const graphics = new Graphics();
                        
                            graphics.lineStyle(1, 0xff0000);
                            graphics.drawRect(
                              chunk_map_entry.x,
                              chunk_map_entry.y,
                              chunk.width * chunk.width,
                              chunk.height * chunk.height
                            );
                        
                            renderer.debugContainer.addChild(graphics);*/

    for (const [i, gid] of chunk.data.entries()) {
      const x =
        (i % this.terrain_params.chunk_size) * this.terrain_params.tile_width;
      const y =
        Math.floor(i / this.terrain_params.chunk_size) *
          this.terrain_params.tile_height +
        this.terrain_params.tile_height;
      const tile_key = get_tile_key(
        layer_kind,
        chunk.position[0],
        chunk.position[1],
        x,
        y,
      );
      if (!this.sprite_by_gid_map[gid]) {
        this.sprite_by_gid_map[gid] = {
          effects: new Set(),
          main_animation_sprite_key: null,
        };
      }
      const tile_effect = this.tile_effects_map[tile_key];
      if (!tile_effect) {
        if (gid === 0) {
          continue;
        }
        this._create_new_tile(
          resource_manager,
          gid,
          x + this.terrain_params.tile_width / 2,
          y - this.terrain_params.tile_height / 2,
          tile_key,
          chunk_map_entry,
        );
      } else {
        if (gid === tile_effect.gid) {
          continue;
        }
        if (!this._active_animations.includes(tile_effect)) {
          this._active_animations.push(tile_effect);
        }

        delete this.tile_effects_map[tile_key];
        this.sprite_by_gid_map[tile_effect.gid].effects.delete(tile_key);

        tile_effect.fade_out.tween.start(window.performance.now());
        tile_effect.fade_out.all_tweens[
          tile_effect.fade_out.all_tweens.length - 1
        ].onComplete(() => {
          chunk_map_entry.container.removeChild(tile_effect.sprite);
        });
        if (gid !== 0) {
          this._create_new_tile(
            resource_manager,
            gid,
            x + this.terrain_params.tile_width / 2,
            y - this.terrain_params.tile_height / 2,
            tile_key,
            chunk_map_entry,
          );
        }
      }
    }
  }

  private _create_new_tile(
    resource_manager: ResourceManager,
    gid: number,
    x: number,
    y: number,
    tile_key: string,
    chunk_map_entry: {
      container: Container;
      data: Chunk;
    },
  ) {
    const graphics = resource_manager.get_graphics_data_by_gid(gid);
    const sprite = resource_manager.get_animated_sprite_from_graphics(graphics);
    const is_animated = graphics.frame_objects.length > 0;

    if (
      is_animated &&
      this.sprite_by_gid_map[gid].main_animation_sprite_key == null
    ) {
      this.sprite_by_gid_map[gid].main_animation_sprite_key = tile_key;
      sprite.play();
    }

    sprite.y = y;
    sprite.rotation = 0;
    this.tile_effects_map[tile_key] = {
      base_props: {
        pos_x: x,
        pos_y: y,
        scale_x: 0,
        scale_y: 0,
        rotation: 0,
        alpha: 1,
      },
      fade_in: create_basic_fade_in_animation(300, Math.random() * 600),
      fade_out: create_basic_fade_out_animation(300, Math.random() * 600),
      sprite,
      gid: gid,
    };
    const new_tile_effect = this.tile_effects_map[tile_key];
    this.sprite_by_gid_map[gid].effects.add(tile_key);
    new_tile_effect.fade_in.tween.start(window.performance.now());
    this._active_animations.push(new_tile_effect);
    chunk_map_entry.container.addChild(sprite);
  }
}

function get_tile_key(
  layer_kind: LayerKind,
  chunk_x: number,
  chunk_y: number,
  tile_x: number,
  tile_y: number,
): string {
  return `${layer_kind}_${chunk_x}_${chunk_y}_${tile_x}_${tile_y}`;
}
