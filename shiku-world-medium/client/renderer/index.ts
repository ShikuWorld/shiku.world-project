import { Container, DisplayObject, IRenderer } from "pixi.js-legacy";
import { LayerName } from "../communication/api/bindings/LayerName";
import { ParallaxContainer } from "./create_game_renderer";
import { Camera } from "@/client/camera";

export interface Point {
  x: number;
  y: number;
}

export type LayerMap = { [keys in LayerName]: Container };

export const createLayerMap: () => LayerMap = () => ({
  BG0: new Container(),
  BG1: new Container(),
  BG2: new Container(),
  BG3: new Container(),
  BG4: new Container(),
  BG5: new Container(),
  BG6: new Container(),
  BG7: new Container(),
  BG8: new Container(),
  BG9: new Container(),
  BG10: new Container(),
  BG11: new Container(),
  GameObjects: new Container(),
  Terrain: new Container(),
  Guest: new Container(),
  Empty: new Container(),
  FG0: new Container(),
  FG1: new Container(),
  FG2: new Container(),
  FG3: new Container(),
  FG4: new Container(),
  FG5: new Container(),
  FG6: new Container(),
  FG7: new Container(),
  FG8: new Container(),
  FG9: new Container(),
  FG10: new Container(),
  FG11: new Container(),
  Menu: new Container(),
});

export const addLayerMapToContainer = (
  container: Container,
  layerMap: LayerMap,
) => {
  container.addChild(
    layerMap.BG0,
    layerMap.BG1,
    layerMap.BG2,
    layerMap.BG3,
    layerMap.BG4,
    layerMap.BG5,
    layerMap.BG6,
    layerMap.BG7,
    layerMap.BG8,
    layerMap.BG9,
    layerMap.BG10,
    layerMap.BG11,
    layerMap.GameObjects,
    layerMap.Terrain,
    layerMap.Guest,
    layerMap.Empty,
    layerMap.FG0,
    layerMap.FG1,
    layerMap.FG2,
    layerMap.FG3,
    layerMap.FG4,
    layerMap.FG5,
    layerMap.FG6,
    layerMap.FG7,
    layerMap.FG8,
    layerMap.FG9,
    layerMap.FG10,
    layerMap.FG11,
    layerMap.Menu,
  );
};

export interface RenderingObject {
  handle: number;
  rotation: number;
  displayObject: DisplayObject;
  pinToCamera: boolean;
}

export type LayerContainer = {
  [key in LayerName]: ParallaxContainer;
};

export interface RenderSystem {
  renderer: IRenderer;
  stage: Container;
  current_main_instance: { instance_id?: string; world_id?: string };
  isDirty: boolean;
}

export interface InstanceRendering {
  mainContainerWrapper: Container;
  mainContainer: Container;
  layerMap: LayerMap;
  camera: Camera;
}
