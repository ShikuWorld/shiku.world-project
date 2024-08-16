import { AnimatedSprite } from "pixi.js";
import {
  create_basic_fade_in_animation,
  create_basic_fade_out_animation,
  SpriteEffect,
  SpriteEffectProperties,
} from "@/client/sprite-animations";
import { update } from "@tweenjs/tween.js";
import { ResourceManager } from "@/client/resources";

type MainSpriteEffects = {
  base_props: SpriteEffectProperties;
  fade_in: SpriteEffect;
  fade_out: SpriteEffect;
  sprite: AnimatedSprite;
  gid: number;
};

type SpriteEffectMap = {
  [key: string]: MainSpriteEffects;
};

export class SpriteEffectsManager {
  private _active_animations: MainSpriteEffects[] = [];
  sprite_by_gid_map: {
    [gid: string]: {
      effects: Set<string>;
      main_animation_sprite_key: string | null;
    };
  } = {};
  sprite_effects_map: SpriteEffectMap = {};

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
      this.sprite_effects_map[tile_key].sprite.textures =
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
      this.sprite_effects_map[
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
        this.sprite_effects_map[sprite_map.main_animation_sprite_key]?.sprite;
      // give role of main sprite to a different sprite if main sprite was removed
      if (!main_sprite) {
        sprite_map.main_animation_sprite_key =
          (sprite_map.effects.values().next().value as string) ?? null;
        main_sprite =
          this.sprite_effects_map[sprite_map.main_animation_sprite_key]?.sprite;
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
        this.sprite_effects_map[tile_key].sprite.currentFrame = current_frame;
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

  add_sprite_with_effects(
    sprite: AnimatedSprite,
    unique_key: string,
    gid: number,
    is_animated: boolean,
  ) {
    if (!this.sprite_by_gid_map[gid]) {
      this.sprite_by_gid_map[gid] = {
        effects: new Set<string>(),
        main_animation_sprite_key: null,
      };
    }
    if (
      is_animated &&
      this.sprite_by_gid_map[gid].main_animation_sprite_key == null
    ) {
      this.sprite_by_gid_map[gid].main_animation_sprite_key = unique_key;
      sprite.play();
    }
    this.sprite_effects_map[unique_key] = {
      base_props: {
        pos_x: sprite.x,
        pos_y: sprite.y,
        scale_x: 0,
        scale_y: 0,
        rotation: 0,
        alpha: 1,
      },
      fade_in: create_basic_fade_in_animation(300, Math.random() * 600),
      fade_out: create_basic_fade_out_animation(300, Math.random() * 600),
      sprite,
      gid,
    };
    const new_tile_effect = this.sprite_effects_map[unique_key];
    this.sprite_by_gid_map[gid].effects.add(unique_key);
    new_tile_effect.fade_in.tween.start(window.performance.now());
    this._active_animations.push(new_tile_effect);
  }

  get_sprite_effect_by_unique_key(
    tile_key: string,
  ): MainSpriteEffects | undefined {
    return this.sprite_effects_map[tile_key];
  }

  remove_sprite_effect(tile_key: string, gid: number) {
    delete this.sprite_effects_map[tile_key];
    this.sprite_by_gid_map[gid].effects.delete(tile_key);
  }
}
