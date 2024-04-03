import {
  addLayerMapToContainer,
  createLayerMap,
  InstanceRendering,
  RenderSystem,
} from "./index";
import {
  autoDetectRenderer,
  Container,
  FederatedMouseEvent,
  Texture,
} from "pixi.js";
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
import { TerrainParams } from "@/client/communication/api/blueprints/TerrainParams";
import { SimpleEventDispatcher } from "strongly-typed-events";

export interface ParallaxContainer extends Container {
  x_pscaling: number;
  y_pscaling: number;
}

export async function create_game_renderer(): Promise<RenderSystem> {
  const canvas_wrapper = document.getElementById("canvas");
  if (!canvas_wrapper) {
    throw new Error("Could not find canvas!");
  }
  const width = canvas_wrapper.offsetWidth;
  const height = canvas_wrapper.offsetHeight;
  const renderer = await autoDetectRenderer({
    backgroundColor: Config.get_bg_color(),
    width,
    height,
  });
  const mainContainer = new Container();
  const global_mouse_position =
    new SimpleEventDispatcher<FederatedMouseEvent>();
  mainContainer.interactive = true;

  mainContainer.on("mousemove", (event) => {
    global_mouse_position.dispatch(event);
  });

  canvas_wrapper.appendChild(renderer.view.canvas as HTMLCanvasElement);

  const dummy_texture_tileset_missing = Texture.from(
    await create_dummy_pic("#ff00ff"),
  );
  const dummy_texture_loading = Texture.from(await create_dummy_pic("#ffff00"));
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
    dummy_texture_tileset_missing,
    dummy_texture_loading,
    current_main_instance: {},
    global_mouse_position,
    stage: mainContainer,
  };
}

const create_dummy_pic = async (color: string): Promise<ImageBitmap> => {
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    console.error("Could not get 2d context...?", canvas);
    return new ImageBitmap();
  }

  canvas.width = 100;
  canvas.height = 100;
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  try {
    return await createImageBitmap(canvas);
  } catch (e) {
    return new ImageBitmap();
  }
};

export const create_instance_rendering = (
  terrain_params: TerrainParams,
): InstanceRendering => {
  const main_container = new Container();
  const main_container_wrapper = new Container();
  const layer_map = createLayerMap();
  addLayerMapToContainer(main_container, layer_map);
  main_container_wrapper.addChild(main_container);
  return {
    camera: create_camera(),
    layer_map,
    main_container,
    main_container_wrapper,
    terrain_params,
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
