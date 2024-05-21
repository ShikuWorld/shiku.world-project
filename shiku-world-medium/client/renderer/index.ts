import {
  Container,
  FederatedMouseEvent,
  Graphics,
  Renderer,
  TilingSprite,
} from "pixi.js";
import { ParallaxContainer } from "./create_game_renderer";
import { Camera } from "@/client/camera";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { TerrainParams } from "@/editor/blueprints/TerrainParams";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { Texture } from "pixi.js";
import { RenderGraphData } from "@/editor/stores/game-instances";

export interface Point {
  x: number;
  y: number;
}

export type LayerMap = { [keys in LayerKind]: ParallaxContainer };

export const createLayerMap: () => LayerMap = () => {
  const map: LayerMap = {
    BG00: new Container() as ParallaxContainer,
    BG01: new Container() as ParallaxContainer,
    BG02: new Container() as ParallaxContainer,
    BG03: new Container() as ParallaxContainer,
    BG04: new Container() as ParallaxContainer,
    BG05: new Container() as ParallaxContainer,
    BG06: new Container() as ParallaxContainer,
    BG07: new Container() as ParallaxContainer,
    BG08: new Container() as ParallaxContainer,
    BG09: new Container() as ParallaxContainer,
    BG10: new Container() as ParallaxContainer,
    ObjectsBelow: new Container() as ParallaxContainer,
    Terrain: new Container() as ParallaxContainer,
    ObjectsFront: new Container() as ParallaxContainer,
    FG00: new Container() as ParallaxContainer,
    FG01: new Container() as ParallaxContainer,
    FG02: new Container() as ParallaxContainer,
    FG03: new Container() as ParallaxContainer,
    FG04: new Container() as ParallaxContainer,
    FG05: new Container() as ParallaxContainer,
    FG06: new Container() as ParallaxContainer,
    FG07: new Container() as ParallaxContainer,
    FG08: new Container() as ParallaxContainer,
    FG09: new Container() as ParallaxContainer,
    FG10: new Container() as ParallaxContainer,
  };
  for (const key of Object.keys(map)) {
    map[key as LayerKind].x_pscaling = 1;
    map[key as LayerKind].y_pscaling = 1;
  }
  return map;
};

export const addLayerMapToContainer = (
  container: Container,
  layerMap: LayerMap,
) => {
  container.addChild(
    layerMap.BG00,
    layerMap.BG01,
    layerMap.BG02,
    layerMap.BG03,
    layerMap.BG04,
    layerMap.BG05,
    layerMap.BG06,
    layerMap.BG07,
    layerMap.BG08,
    layerMap.BG09,
    layerMap.BG10,
    layerMap.ObjectsBelow,
    layerMap.Terrain,
    layerMap.ObjectsFront,
    layerMap.FG00,
    layerMap.FG01,
    layerMap.FG02,
    layerMap.FG03,
    layerMap.FG04,
    layerMap.FG05,
    layerMap.FG06,
    layerMap.FG07,
    layerMap.FG08,
    layerMap.FG09,
    layerMap.FG10,
  );
};

export interface RenderingObject {
  handle: number;
  rotation: number;
  displayObject: Container;
  pinToCamera: boolean;
}

export type LayerContainer = {
  [key in LayerKind]: ParallaxContainer;
};

export interface RenderSystem {
  renderer: Renderer;
  stage: Container;
  blueprint_render_data: RenderGraphData | null;
  dummy_texture_tileset_missing: Texture;
  dummy_texture_loading: Texture;
  current_main_instance: { instance_id?: string; world_id?: string };
  global_mouse_position: SimpleEventDispatcher<FederatedMouseEvent>;
  isDirty: boolean;
}

export interface InstanceRendering {
  main_container_wrapper: Container;
  main_container: Container;
  blueprint_container: Container;
  grid?: {
    sprite: TilingSprite;
    selected_tile: Graphics;
    grid_container: Container;
    scaling: { x: number; y: number };
    last_mouse_move_position: { x: number; y: number };
  };
  layer_map: LayerMap;
  camera: Camera;
  terrain_params: TerrainParams;
}
