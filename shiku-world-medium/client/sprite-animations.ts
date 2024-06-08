import { Easing, Tween } from "@tweenjs/tween.js";

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

export function create_basic_fade_in_animation(
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
    .to({ scale_x: 1.5, scale_y: 1.5 }, duration * (1 / 2))
    .easing(Easing.Quadratic.Out)
    .delay(delay);
  const tween_2 = new Tween(add_props)
    .to({ scale_x: 1, scale_y: 1 }, duration * (1 / 2))
    .easing(Easing.Quadratic.In);
  return {
    add_props,
    tween: tween_1.chain(tween_2),
    all_tweens: [tween_1, tween_2],
  };
}

export function create_basic_fade_out_animation(
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
