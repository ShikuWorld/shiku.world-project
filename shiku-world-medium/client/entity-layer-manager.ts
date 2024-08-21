import { LayerKind } from "@/editor/blueprints/LayerKind";
import { Container } from "pixi.js";
import { LayerMap, PossibleLayers } from "@/client/renderer";

type EntityLayerMap = { [keys in LayerKind]: Container };

export class EntityLayerManager {
  private _entity_layer_map: EntityLayerMap;
  private _container_map: {
    [entity_id: string | number]: {
      layer: LayerKind;
      container: Container;
      display_object: Container;
    };
  } = {};

  constructor(create_container: () => Container) {
    this._entity_layer_map = PossibleLayers.reduce((acc, key) => {
      acc[key] = create_container();
      return acc;
    }, {} as EntityLayerMap);
  }

  get_entity(entity_id: string | number) {
    return this._container_map[entity_id];
  }

  attach_to_layer_map(layer_map: LayerMap) {
    for (const key of Object.keys(layer_map)) {
      layer_map[key as LayerKind].addChild(
        this._entity_layer_map[key as LayerKind],
      );
    }
  }

  add_display_object(
    entity_id: string | number,
    layer: LayerKind,
    display_object: Container,
    create_container: () => Container,
  ) {
    this._container_map[entity_id] = {
      layer,
      container: create_container(),
      display_object,
    };
    this._container_map[entity_id].container.addChild(display_object);
    this._entity_layer_map[layer].addChild(
      this._container_map[entity_id].container,
    );
  }

  remove_display_object(entity_id: string | number) {
    if (this._container_map[entity_id]) {
      const { layer, container } = this._container_map[entity_id];
      this._entity_layer_map[layer].removeChild(container);
      delete this._container_map[entity_id];
      container.destroy();
    }
  }

  move_display_object_between_layers(
    entity_id: string | number,
    from_layer: LayerKind,
    to_layer: LayerKind,
  ) {
    if (!this._container_map[entity_id]) {
      return;
    }
    const { container } = this._container_map[entity_id];
    this._entity_layer_map[from_layer].removeChild(container);
    this._entity_layer_map[to_layer].addChild(container);
  }

  clear(layer_map: LayerMap) {
    for (const key of Object.keys(layer_map)) {
      layer_map[key as LayerKind].removeChild(
        this._entity_layer_map[key as LayerKind],
      );
    }
    this._container_map = {};
  }

  update_container_display_object(
    entity_id: string | number,
    display_object: Container,
  ) {
    this._container_map[entity_id].display_object = display_object;
    this._container_map[entity_id].container.removeChildren();
    this._container_map[entity_id].container.addChild(display_object);
  }

  update_container_position(renderKey: number | string, container: Container) {
    if (this._container_map[renderKey]) {
      const new_pos = container.toGlobal({ x: 0, y: 0 });
      this._container_map[renderKey].container.position.copyFrom(new_pos);
      this._container_map[renderKey].container.zIndex = new_pos.y;
    }
  }

  change_layer_transparency(number: number) {
    for (const key of Object.keys(this._entity_layer_map)) {
      this._entity_layer_map[key as LayerKind].alpha = number;
    }
  }
}
