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

export const PossibleLayers = [
  "BG00",
  "BG01",
  "BG02",
  "BG03",
  "BG04",
  "BG05",
  "BG06",
  "BG07",
  "BG08",
  "BG09",
  "BG10",
  "ObjectsBelow",
  "Terrain",
  "ObjectsFront",
  "FG00",
  "FG01",
  "FG02",
  "FG03",
  "FG04",
  "FG05",
  "FG06",
  "FG07",
  "FG08",
  "FG09",
  "FG10",
] as const;

export const createLayerMap: () => LayerMap = () => {
  const map: LayerMap = PossibleLayers.reduce((acc, key) => {
    acc[key] = new Container() as ParallaxContainer;
    acc[key].x_pscaling = 1;
    acc[key].y_pscaling = 1;
    return acc;
  }, {} as LayerMap);
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
    mouse_wheel_event: SimpleEventDispatcher<number>;
    p_scaling: { x: number; y: number };
    last_mouse_move_position: { x: number; y: number };
  };
  layer_map: LayerMap;
  camera: Camera;
  terrain_params: TerrainParams;
}
