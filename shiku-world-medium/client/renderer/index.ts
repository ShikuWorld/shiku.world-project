import { Container, DisplayObject, IRenderer } from "pixi.js-legacy";
import { LayerName } from "../communication/api/bindings/LayerName";
import { Layer } from "@pixi/layers";
import { ParallaxContainer } from "./create_game_renderer";

export interface Point {
  x: number;
  y: number;
}

export const worldLayerMap: { [keys in LayerName]: Layer } = {
  BG0: new Layer(),
  BG1: new Layer(),
  BG2: new Layer(),
  BG3: new Layer(),
  BG4: new Layer(),
  BG5: new Layer(),
  BG6: new Layer(),
  BG7: new Layer(),
  BG8: new Layer(),
  BG9: new Layer(),
  BG10: new Layer(),
  BG11: new Layer(),
  GameObjects: new Layer(),
  Terrain: new Layer(),
  Guest: new Layer(),
  Empty: new Layer(),
  FG0: new Layer(),
  FG1: new Layer(),
  FG2: new Layer(),
  FG3: new Layer(),
  FG4: new Layer(),
  FG5: new Layer(),
  FG6: new Layer(),
  FG7: new Layer(),
  FG8: new Layer(),
  FG9: new Layer(),
  FG10: new Layer(),
  FG11: new Layer(),
  Menu: new Layer(),
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
  mainContainer: Container;
  isDirty: boolean;
}
