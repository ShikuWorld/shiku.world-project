import {
  AnimatedSprite,
  BitmapText,
  Container,
  DisplayObject,
  Sprite,
  WRAP_MODES,
} from "pixi.js-legacy";
import { InstanceRendering } from "../renderer";
import { ShowEntity } from "../communication/api/bindings/ShowEntity";
import { Graphics, ResourceManager } from "../resources";
import { UpdateEntity } from "../communication/api/bindings/UpdateEntity";
import { EntityRenderData } from "../communication/api/bindings/EntityRenderData";
import { match, P } from "ts-pattern";
import { Config } from "../config";
import { create_countdown } from "../countdown";
import { RemoveEntity } from "../communication/api/bindings/RemoveEntity";
import { StaticImage } from "../communication/api/bindings/StaticImage";
import { SimpleImageEffect } from "../communication/api/bindings/SimpleImageEffect";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { RenderGraph } from "@/client/render-graph";

export function create_entity_manager(): EntityManager {
  return new EntityManager();
}

export function create_render_graph(): RenderGraph {
  return new RenderGraph();
}

export class EntityManager {
  private _entity_map: Map<string, Entity>;
  constructor() {
    this._entity_map = new Map();
  }

  iterate_entities(cb: (entity: Entity) => void) {
    for (const entity of this._entity_map.values()) {
      cb(entity);
    }
  }

  get_entity(entity_id: string): Entity | undefined {
    const entity = this._entity_map.get(entity_id);
    if (!entity) {
      console.warn(
        `Could not get entity with id ${entity_id}, reason: didn't exist`,
      );
      return;
    }

    return entity;
  }

  add_simple_image_effect(
    simple_image_effect: SimpleImageEffect,
    renderer: InstanceRendering,
    resource_manager: ResourceManager,
  ) {
    const [container, layer_name] = get_display_obj_from_render(
      resource_manager,
      {
        StaticImage: {
          tiled: false,
          height: null,
          width: null,
          graphic_id: simple_image_effect.graphic_id,
          layer: simple_image_effect.layer,
          scale: [1.0, 1.0],
          blending_mode: simple_image_effect.blending_mode,
          offset_2d: [0, 0],
        },
      },
      renderer,
    );

    if (!layer_name) {
      return;
    }

    const isometry = {
      x: simple_image_effect.initial_isometrics_2d[0],
      y: simple_image_effect.initial_isometrics_2d[1],
      rotation: simple_image_effect.initial_isometrics_2d[2],
    };

    container.x = Math.round(isometry.x * Config.get_simulation_scale());
    container.y = Math.round(isometry.y * Config.get_simulation_scale());
    container.rotation = isometry.rotation;

    const parent = simple_image_effect.parent_entity
      ? this._entity_map.get(simple_image_effect.parent_entity)
      : undefined;
    const display_object = container.getChildAt(0);
    if (display_object instanceof AnimatedSprite) {
      const graphics = resource_manager.get_graphics_data_by_gid(
        Number(simple_image_effect.graphic_id),
      );
      display_object.loop = false;
      display_object.alpha = simple_image_effect.transparency;

      const animation_length = graphics.frame_objects.reduce(
        (acc, f_o) => acc + f_o.time,
        0,
      );

      if (parent) {
        parent.wrapper.addChild(container);
      } else {
        renderer.layer_map[layer_name].addChild(container);
      }

      setTimeout(() => {
        if (parent) {
          parent.wrapper.removeChild(container);
        } else {
          renderer.layer_map[layer_name].removeChild(container);
        }
      }, animation_length);
    }
  }

  add_entity(
    show_entity: ShowEntity,
    renderer: InstanceRendering,
    resource_manager: ResourceManager,
  ) {
    if (this._entity_map.has(show_entity.id)) {
      return;
    }

    const [container, layer_name] = get_display_obj_from_render(
      resource_manager,
      show_entity.render,
      renderer,
    );

    if (!layer_name) {
      return;
    }

    const isometry = {
      x: show_entity.initial_isometrics_2d[0],
      y: show_entity.initial_isometrics_2d[1],
      rotation: show_entity.initial_isometrics_2d[2],
    };

    container.x = Math.round(isometry.x * Config.get_simulation_scale());
    container.y = Math.round(isometry.y * Config.get_simulation_scale());
    container.rotation = isometry.rotation;

    const parent = show_entity.parent_entity
      ? this._entity_map.get(show_entity.parent_entity)
      : undefined;
    if (parent) {
      parent.wrapper.addChild(container);

      this._entity_map.set(show_entity.id, {
        layer_name,
        id: show_entity.id,
        isometry,
        render: show_entity.render,
        parent_container: parent.wrapper,
        wrapper: container,
      });
    } else {
      renderer.layer_map[layer_name].addChild(container);

      this._entity_map.set(show_entity.id, {
        layer_name,
        id: show_entity.id,
        isometry,
        render: show_entity.render,
        parent_container: renderer.layer_map[layer_name],
        wrapper: container,
      });
    }
  }

  update_entity_position(
    entity_id: string,
    x: number,
    y: number,
    rotation: number,
  ) {
    const entity = this.get_entity(entity_id);
    if (!entity) {
      return;
    }

    entity.isometry.x = x;
    entity.isometry.y = y;
    entity.isometry.rotation = rotation;

    entity.wrapper.x = Math.round(
      entity.isometry.x * Config.get_simulation_scale(),
    );
    entity.wrapper.y = Math.round(
      entity.isometry.y * Config.get_simulation_scale(),
    );
    entity.wrapper.rotation = entity.isometry.rotation;
  }

  update_entity(
    update_entity: UpdateEntity,
    resource_manager: ResourceManager,
    _renderer: InstanceRendering,
  ) {
    const entity = this.get_entity(update_entity.id);
    if (!entity) {
      return;
    }

    if ("RenderTypeText" in update_entity.render) {
      (entity.wrapper.getChildAt(0) as BitmapText).text =
        update_entity.render.RenderTypeText.text;
      return;
    }

    if (
      "RenderTypeTimer" in update_entity.render ||
      "NoRender" in update_entity.render
    ) {
      return;
    }

    let graphic_id = -1;
    if ("StaticImage" in update_entity.render) {
      graphic_id = Number(update_entity.render.StaticImage.graphic_id);
    }
    const graphics = resource_manager.get_graphics_data_by_gid(graphic_id);

    if (!graphics) {
      return;
    }

    const display_object = entity.wrapper.getChildAt(0);
    if (!(display_object instanceof Sprite)) {
      console.log("Was not instanceof sprite");
      return;
    }

    const was_animated_sprite = display_object instanceof AnimatedSprite;
    const is_animated_sprite = graphics.frame_objects.length > 0;

    if (!is_animated_sprite && !was_animated_sprite) {
      display_object.texture = graphics.textures[0];
    } else {
      EntityManager._recreate_sprite(entity, graphics);
    }

    const sprite = entity.wrapper.getChildAt(0) as Sprite;
    set_sprite_props_from_static_image_data(
      sprite,
      update_entity.render.StaticImage,
    );
  }

  private static _recreate_sprite(entity: Entity, graphics: Graphics) {
    entity.wrapper.removeChildAt(0);
    entity.wrapper.addChildAt(
      get_sprite_from_render(
        (entity.render as { StaticImage: StaticImage }).StaticImage,
        graphics,
      ),
      0,
    );
  }

  remove_entity(remove_entity: RemoveEntity) {
    const entity = this._entity_map.get(remove_entity.id);
    if (!entity) {
      throw Error(
        `Could not remove entity with id ${remove_entity.id}, reason: didn't exist`,
      );
    }

    entity.parent_container.removeChild(entity.wrapper);
    this._entity_map.delete(remove_entity.id);
  }

  remove_all_entities_from_module() {
    for (const entity of this._entity_map.values()) {
      entity.parent_container.removeChild(entity.wrapper);
      this._entity_map.delete(entity.id);
    }
  }
}

function set_sprite_props_from_static_image_data(
  sprite: Sprite,
  staticImageData: StaticImage,
) {
  sprite.x = -Math.round(staticImageData.offset_2d[0]);
  sprite.y = -Math.round(staticImageData.offset_2d[1]);
  sprite.scale.x = staticImageData.scale[0];
  sprite.scale.y = staticImageData.scale[1];
  if (staticImageData.tiled) {
    sprite.texture.baseTexture.wrapMode = WRAP_MODES.REPEAT;
  }
  if (staticImageData.width) {
    sprite.width = staticImageData.width;
  }
  if (staticImageData.height) {
    sprite.height = staticImageData.height;
  }
}

function get_sprite_from_render(
  staticImageData: StaticImage,
  graphics: Graphics,
): DisplayObject {
  let sprite: Sprite | AnimatedSprite;

  if (graphics.frame_objects.length > 0) {
    const animated_sprite = new AnimatedSprite(graphics.frame_objects);
    animated_sprite.play();
    sprite = animated_sprite;
  } else {
    sprite = new Sprite(graphics.textures[0]);
  }

  set_sprite_props_from_static_image_data(sprite, staticImageData);

  sprite.anchor.set(0.5);

  if (staticImageData.blending_mode) {
    sprite.blendMode = staticImageData.blending_mode;
  }

  return sprite;
}

function get_display_obj_from_render(
  resource_manager: ResourceManager,
  render: EntityRenderData,
  _renderer: InstanceRendering,
): [Container, LayerKind | undefined] {
  const container = new Container();
  let layer_name: LayerKind | undefined = undefined;
  container.addChild(
    match(render)
      .with({ StaticImage: P.select() }, (staticImageData) => {
        layer_name = staticImageData.layer as LayerKind;
        const graphics = resource_manager.get_graphics_data_by_gid(
          Number(staticImageData.graphic_id),
        );

        return get_sprite_from_render(staticImageData, graphics);
      })
      .with({ RenderTypeTimer: P.select() }, (data) => {
        layer_name = data.layer as LayerKind;

        return create_countdown(data);
      })
      .with({ RenderTypeText: P.select() }, (data) => {
        layer_name = data.layer as LayerKind;
        const text = new BitmapText(data.text, { fontName: data.font_family });

        if (data.center_x) {
          text.position.x = Math.round(-text.textWidth / 2);
        }
        return text;
      })
      .with({ NoRender: P.select() }, (_data) => {
        return new Sprite();
      })
      .exhaustive(),
  );

  return [container, layer_name];
}

export interface Renderable {
  isometry: Isometry;
  parent_container: Container;
  wrapper: Container;
}

export interface Entity extends Renderable {
  id: string;
  layer_name: LayerKind;
  render: EntityRenderData;
}

export interface Isometry {
  x: number;
  y: number;
  rotation: number;
}
