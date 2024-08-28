import { Easing, Tween } from "@tweenjs/tween.js";
import { FadeinEffect } from "@/editor/blueprints/FadeinEffect";
import { match } from "ts-pattern";
import { FadeoutEffect } from "@/editor/blueprints/FadeoutEffect";

export type SpriteEffectProperties = {
  pos_x: number;
  pos_y: number;
  scale_x: number;
  scale_y: number;
  rotation: number;
  alpha: number;
};
export type SpriteEffect = {
  add_props: SpriteEffectProperties;
  tween: Tween<SpriteEffectProperties>;
  all_tweens: Tween<SpriteEffectProperties>[];
};

export function create_no_effect(
  _duration: number,
  _delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: 0,
    scale_x: 0,
    scale_y: 0,
    rotation: 0,
    alpha: 0,
  };
  const tween = new Tween(add_props).to({ scale_x: 0.0 }, 0).delay(0);
  return {
    add_props,
    tween: tween,
    all_tweens: [tween],
  };
}

export function create_simple_fade_in(
  duration: number,
  delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: 0,
    scale_x: 0,
    scale_y: 0,
    rotation: 0,
    alpha: -1,
  };
  const tween = new Tween(add_props)
    .easing(Easing.Cubic.In)
    .to({ alpha: 0.0 }, duration)
    .delay(delay);
  return {
    add_props,
    tween: tween,
    all_tweens: [tween],
  };
}

export function create_simple_fade_out(
  duration: number,
  delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: 0,
    scale_x: 0,
    scale_y: 0,
    rotation: 0,
    alpha: 0,
  };
  const tween = new Tween(add_props)
    .easing(Easing.Quadratic.Out)
    .to({ alpha: -1.0 }, duration)
    .delay(delay);
  return {
    add_props,
    tween: tween,
    all_tweens: [tween],
  };
}

export function create_tile_fade_in_animation(
  duration: number,
  delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: 0,
    scale_x: -1,
    scale_y: -1,
    rotation: 0,
    alpha: 0,
  };
  const tween_1 = new Tween(add_props)
    .to({ scale_x: 1.0, scale_y: 1.0 }, duration * (1 / 2))
    .easing(Easing.Quadratic.Out)
    .delay(delay);
  const tween_2 = new Tween(add_props)
    .to({ scale_x: 0, scale_y: 0 }, duration * (1 / 2))
    .easing(Easing.Quadratic.In);
  return {
    add_props,
    tween: tween_1.chain(tween_2),
    all_tweens: [tween_1, tween_2],
  };
}

export function create_fade_in_animation(
  fade_in_effect: FadeinEffect,
  duration: number,
): SpriteEffect {
  return match(fade_in_effect)
    .with("None", () => create_no_effect(duration, 0))
    .with("Fade", () => create_simple_fade_in(duration, 0))
    .with("JumpForth", () => create_jump_forth_animation(duration, 0))
    .exhaustive();
}

export function create_fade_out_animation(
  fade_out_effect: FadeoutEffect,
  duration: number,
): SpriteEffect {
  return match(fade_out_effect)
    .with("None", () => create_no_effect(duration, 0))
    .with("Fade", () => create_simple_fade_out(duration, 0))
    .with("JumpBack", () => create_jump_back_animation(duration, 0))
    .exhaustive();
}

export function create_jump_back_animation(
  duration: number,
  delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: 0,
    scale_x: 0,
    scale_y: 0,
    rotation: 0,
    alpha: 0,
  };
  const tween_1 = new Tween(add_props)
    .to(
      { pos_y: -80, alpha: -0.5, scale_x: -0.5, scale_y: -0.5 },
      duration * (1 / 2),
    )
    .easing(Easing.Quadratic.In)
    .delay(delay);
  const tween_2 = new Tween(add_props)
    .to(
      { pos_y: -30, scale_x: -0.8, scale_y: -0.8, alpha: -1 },
      duration * (1 / 2),
    )
    .easing(Easing.Quadratic.Out);
  return {
    add_props,
    tween: tween_1.chain(tween_2),
    all_tweens: [tween_1, tween_2],
  };
}

export function create_jump_forth_animation(
  duration: number,
  delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: -30,
    scale_x: -0.8,
    scale_y: -0.8,
    rotation: 0,
    alpha: -1,
  };
  const tween_1 = new Tween(add_props)
    .to(
      { pos_y: -80, alpha: -0.5, scale_x: -0.5, scale_y: -0.5 },
      duration * (1 / 2),
    )
    .easing(Easing.Quadratic.Out)
    .delay(delay);
  const tween_2 = new Tween(add_props)
    .to({ pos_y: 0, scale_x: 0, scale_y: 0, alpha: 0 }, duration * (1 / 2))
    .easing(Easing.Quadratic.In);
  return {
    add_props,
    tween: tween_1.chain(tween_2),
    all_tweens: [tween_1, tween_2],
  };
}

export function create_tile_fade_out_animation(
  duration: number,
  delay: number,
): SpriteEffect {
  const add_props: SpriteEffectProperties = {
    pos_x: 0,
    pos_y: 0,
    scale_x: 0,
    scale_y: 0,
    rotation: 0,
    alpha: 0,
  };
  const tween_1 = new Tween(add_props)
    .to({ scale_x: 0.3, scale_y: 0.3 }, duration * (1 / 2))
    .easing(Easing.Quadratic.In)
    .delay(delay);
  const tween_2 = new Tween(add_props)
    .to({ scale_x: -1, scale_y: -1 }, duration * (1 / 2))
    .easing(Easing.Quadratic.Out);
  return {
    add_props,
    tween: tween_1.chain(tween_2),
    all_tweens: [tween_1, tween_2],
  };
}
