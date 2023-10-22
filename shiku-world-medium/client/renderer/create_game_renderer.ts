import { LayerContainer, worldLayerMap, Renderer } from "./index";
import { autoDetectRenderer, CanvasRenderer, Container } from "pixi.js-legacy";
import { Config } from "../config";
import {
  get_camera_zoom,
  get_enable_zoom,
  get_stage_height,
  get_stage_width,
  set_stage_height,
  set_stage_width,
} from "../config/config";
import { Cull } from "@pixi-essentials/cull";
import { applyCanvasMixin, Stage } from "@pixi/layers";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { LayerName } from "../communication/api/bindings/LayerName";

export interface ParallaxContainer extends Container {
  x_pscaling: number;
  y_pscaling: number;
}

applyCanvasMixin(CanvasRenderer);
export function create_game_renderer(): Renderer {
  const canvas_wrapper = document.getElementById("canvas");
  const width = canvas_wrapper.offsetWidth;
  const height = canvas_wrapper.offsetHeight;
  const renderer = autoDetectRenderer({
    width,
    height,
  });

  renderer.backgroundColor = Config.get_bg_color();

  const worldContainer = new Container();
  const stage = new Stage();
  const layerContainer: Partial<LayerContainer> = {};
  stage.sortableChildren = true;

  stage.addChild(worldContainer);

  const cull = new Cull({ recursive: true });

  for (const key in worldLayerMap) {
    layerContainer[key as LayerName] = new Container() as ParallaxContainer;
    layerContainer[key as LayerName].x_pscaling = 1;
    layerContainer[key as LayerName].y_pscaling = 1;
    worldContainer.addChild(layerContainer[key as LayerName]);
  }

  cull.add(layerContainer.Terrain);
  cull.add(layerContainer.BG1);
  cull.add(layerContainer.BG2);
  cull.add(layerContainer.BG3);
  cull.add(layerContainer.BG4);
  cull.add(layerContainer.BG5);
  cull.add(layerContainer.BG6);
  cull.add(layerContainer.BG7);
  cull.add(layerContainer.BG8);
  cull.add(layerContainer.BG9);
  cull.add(layerContainer.BG10);
  cull.add(layerContainer.BG11);

  document.getElementById("canvas").appendChild(renderer.view);

  const renderingSystem: Renderer = {
    renderer,
    stage,
    isDirty: true,
    worldContainer,
    onStageResize: new SimpleEventDispatcher(),
    layerContainer: layerContainer as LayerContainer,
    renderingObjects: [],
  };

  setup_resizing(renderingSystem);

  renderer.on("prerender", () => {
    renderingSystem.layerContainer.GameObjects.children.sort((a, b) => {
      if (a.y > b.y) {
        return 1;
      } else if (a.y < b.y) {
        return -1;
      }

      return 0;
    });

    if (renderingSystem.isDirty) {
      cull.cull(renderer.screen);
      renderingSystem.isDirty = false;
    }
  });

  return renderingSystem;
}

export const viewPortResize = (
  width: number,
  height: number,
  renderer: Renderer
) => {
  renderer.renderer.view.style.width = `${width}px`;
  renderer.renderer.view.style.height = `${height}px`;

  if (get_enable_zoom()) {
    set_stage_width(width * get_camera_zoom());
    set_stage_height(height * get_camera_zoom());
  } else {
    set_stage_width(width * window.devicePixelRatio * get_camera_zoom());
    set_stage_height(height * window.devicePixelRatio * get_camera_zoom());
  }

  renderer.renderer.resize(get_stage_width(), get_stage_height());
  setTimeout(() => {
    renderer.isDirty = true;
  }, 50);
  renderer.onStageResize.dispatch({
    stage_width: Config.get_stage_width(),
    stage_height: Config.get_stage_height(),
  });
};

const setup_resizing = (renderer: Renderer) => {
  const canvas_wrapper = document.getElementById("canvas");

  const twitch_chat = document.getElementById("twitch-chat") || {
    className: "",
    clientWidth: 0,
    style: { width: "" },
  };
  twitch_chat.className = "";

  window.addEventListener("resize", () => {
    viewPortResize(
      canvas_wrapper.offsetWidth,
      canvas_wrapper.offsetHeight,
      renderer
    );
  });
  viewPortResize(
    canvas_wrapper.offsetWidth,
    canvas_wrapper.offsetHeight,
    renderer
  );

  const toggle_chat_element = document.getElementById("toggle-chat");
  if (toggle_chat_element) {
    toggle_chat_element.addEventListener("click", (e) => {
      const toggle = e.target as HTMLElement;
      if (toggle.innerHTML === "»") {
        twitch_chat.style.width = "0px";
        toggle.innerHTML = "«";
      } else {
        twitch_chat.style.width = null;
        toggle.innerHTML = "»";
      }
      viewPortResize(
        canvas_wrapper.offsetWidth,
        canvas_wrapper.offsetHeight,
        renderer
      );
    });
  }
};
