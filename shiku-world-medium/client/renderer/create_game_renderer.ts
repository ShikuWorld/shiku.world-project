import {
  addLayerMapToContainer,
  createLayerMap,
  InstanceRendering,
  RenderSystem,
} from "./index";
import { autoDetectRenderer, Container } from "pixi.js-legacy";
import { Config } from "../config";
import {
  get_camera_zoom,
  get_enable_zoom,
  get_stage_height,
  get_stage_width,
  set_stage_height,
  set_stage_width,
} from "../config/config";
import { create_camera } from "@/client/camera";

export interface ParallaxContainer extends Container {
  x_pscaling: number;
  y_pscaling: number;
}

export function create_game_renderer(): RenderSystem {
  const canvas_wrapper = document.getElementById("canvas");
  if (!canvas_wrapper) {
    throw new Error("Could not find canvas!");
  }
  const width = canvas_wrapper.offsetWidth;
  const height = canvas_wrapper.offsetHeight;
  const renderer = autoDetectRenderer({
    backgroundColor: Config.get_bg_color(),
    width,
    height,
  });
  const mainContainer = new Container();
  canvas_wrapper.appendChild(renderer.view as HTMLCanvasElement);

  //setup_resizing(renderingSystem);

  /*renderer.on("prerender", () => {
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
  });*/

  return {
    renderer,
    isDirty: true,
    current_main_instance: {},
    stage: mainContainer,
  };
}

export const create_instance_rendering = (): InstanceRendering => {
  const mainContainer = new Container();
  const mainContainerWrapper = new Container();
  const layerMap = createLayerMap();
  addLayerMapToContainer(mainContainer, layerMap);
  mainContainerWrapper.addChild(mainContainer);
  return {
    camera: create_camera(),
    layerMap,
    mainContainer,
    mainContainerWrapper,
  };
};

export const viewPortResize = (
  width: number,
  height: number,
  renderer: RenderSystem,
) => {
  /*renderer.renderer.view.style.width = `${width}px`;
  renderer.renderer.view.style.height = `${height}px`;*/

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
  /*renderer.onStageResize.dispatch({
    stage_width: Config.get_stage_width(),
    stage_height: Config.get_stage_height(),
  });*/
};

/*
const setup_resizing = (_renderer: Renderer) => {
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
      renderer,
    );
  });
  viewPortResize(
    canvas_wrapper.offsetWidth,
    canvas_wrapper.offsetHeight,
    renderer,
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
        renderer,
      );
    });
  }
};
*/
