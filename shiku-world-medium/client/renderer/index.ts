import { Container, DisplayObject, IRenderer } from "pixi.js-legacy";
import { ParallaxContainer } from "./create_game_renderer";
import { Camera } from "@/client/camera";
import { LayerKind } from "@/editor/blueprints/LayerKind";

export interface Point {
  x: number;
  y: number;
}

export type LayerMap = { [keys in LayerKind]: Container };

export const createLayerMap: () => LayerMap = () => ({
  BG00: new Container(),
  BG01: new Container(),
  BG02: new Container(),
  BG03: new Container(),
  BG04: new Container(),
  BG05: new Container(),
  BG06: new Container(),
  BG07: new Container(),
  BG08: new Container(),
  BG09: new Container(),
  BG10: new Container(),
  ObjectsBelow: new Container(),
  Terrain: new Container(),
  ObjectsFront: new Container(),
  FG00: new Container(),
  FG01: new Container(),
  FG02: new Container(),
  FG03: new Container(),
  FG04: new Container(),
  FG05: new Container(),
  FG06: new Container(),
  FG07: new Container(),
  FG08: new Container(),
  FG09: new Container(),
  FG10: new Container(),
});

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
  displayObject: DisplayObject;
  pinToCamera: boolean;
}

export type LayerContainer = {
  [key in LayerKind]: ParallaxContainer;
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
