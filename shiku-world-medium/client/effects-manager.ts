import { AnimatedSprite } from "pixi.js";
import {
  SpriteEffect,
  SpriteEffectProperties,
} from "@/client/sprite-animations";
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

export class EffectsManager {
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
      if (!this.sprite_effects_map[effects_for_gid.main_animation_sprite_key]) {
        // No main animation sprite key set
        return;
      }
      this.sprite_effects_map[
        effects_for_gid.main_animation_sprite_key
      ].sprite.loop = graphics.loop_animation !== false;
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
      tile_effect.fade_in.all_tweens.forEach((t) =>
        t.update(window.performance.now()),
      );
      tile_effect.fade_out.all_tweens.forEach((t) =>
        t.update(window.performance.now()),
      );
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
    fade_in_effect: SpriteEffect,
    fade_out_effect: SpriteEffect,
  ) {
    if (!this.sprite_by_gid_map[gid]) {
      this.sprite_by_gid_map[gid] = {
        effects: new Set<string>(),
        main_animation_sprite_key: null,
      };
    }
    const is_animated = sprite.textures.length > 1;
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
        scale_x: 1,
        scale_y: 1,
        rotation: sprite.rotation,
        alpha: 1,
      },
      fade_in: fade_in_effect,
      fade_out: fade_out_effect,
      sprite,
      gid,
    };
    const new_sprite_effect = this.sprite_effects_map[unique_key];
    this.sprite_by_gid_map[gid].effects.add(unique_key);
    new_sprite_effect.fade_in.tween.start(window.performance.now());
    this._active_animations.push(new_sprite_effect);
  }

  get_sprite_effect_by_unique_key(
    tile_key: string,
  ): MainSpriteEffects | undefined {
    return this.sprite_effects_map[tile_key];
  }

  start_fade_out_animation(
    tile_key: string,
    on_complete: () => void = () => {},
  ) {
    const sprite_effect = this.sprite_effects_map[tile_key];
    this.remove_sprite_effect(tile_key);
    if (!sprite_effect) {
      console.error("Could not find sprite effect to remove?!", tile_key);
    } else {
      this._active_animations.push(sprite_effect);
      sprite_effect.fade_out.tween.start(window.performance.now());
      sprite_effect.fade_out.all_tweens[
        sprite_effect.fade_out.all_tweens.length - 1
      ].onComplete(() => {
        on_complete();
      });
    }
  }

  remove_sprite_effect(unique_key: string) {
    const sprite_effect = this.sprite_effects_map[unique_key];
    if (!sprite_effect) {
      console.error(
        "Could not find sprite effect to remove?!",
        unique_key,
        this.sprite_effects_map,
      );
    } else {
      const gid = sprite_effect.gid;
      if (this.sprite_by_gid_map[gid]) {
        delete this.sprite_effects_map[unique_key];
        this.sprite_by_gid_map[gid].effects.delete(unique_key);
      } else {
        console.error(
          "Trying to remove a sprite that is not the main animation sprite",
        );
      }
    }
  }

  update_sprite(unique_key: string, sprite: AnimatedSprite, new_gid: number) {
    const sprite_effect = this.sprite_effects_map[unique_key];
    if (!sprite_effect) {
      console.error(
        "Could not find sprite effect to update?!",
        unique_key,
        this.sprite_effects_map,
      );
    } else {
      const current_gid = sprite_effect.gid;
      if (new_gid === current_gid) {
        return;
      }
      if (!this.sprite_by_gid_map[new_gid]) {
        this.sprite_by_gid_map[new_gid] = {
          effects: new Set<string>(),
          main_animation_sprite_key: null,
        };
      }
      if (this.sprite_by_gid_map[current_gid]) {
        this.sprite_by_gid_map[current_gid].effects.delete(unique_key);
        if (this.sprite_by_gid_map[current_gid].effects.size === 0) {
          delete this.sprite_by_gid_map[current_gid];
        } else {
          if (
            this.sprite_by_gid_map[current_gid].main_animation_sprite_key ===
            unique_key
          ) {
            const next_unique_key = this.sprite_by_gid_map[current_gid].effects
              .values()
              .next().value;
            this.sprite_by_gid_map[current_gid].main_animation_sprite_key =
              next_unique_key;
            const next_sprite = this.sprite_effects_map[next_unique_key].sprite;
            next_sprite.play();
          }
        }
        sprite_effect.gid = new_gid;
        sprite_effect.sprite = sprite;
        const is_animated = sprite.textures.length > 1;
        this.sprite_by_gid_map[new_gid].effects.add(unique_key);
        if (
          is_animated &&
          this.sprite_by_gid_map[new_gid].main_animation_sprite_key == null
        ) {
          this.sprite_by_gid_map[new_gid].main_animation_sprite_key =
            unique_key;
          sprite.play();
        }
      } else {
        console.error(
          "Trying to remove a sprite that is not the main animation sprite",
        );
      }
    }
  }
}
