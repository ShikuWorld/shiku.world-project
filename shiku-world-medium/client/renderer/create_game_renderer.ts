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
import { create_camera } from "@/client/camera";
import { SimpleEventDispatcher } from "strongly-typed-events";
import { GameInstancesStore } from "@/editor/stores/game-instances";
import { GameInstanceMap } from "@/client/game-instance";
import { WorldParams } from "@/editor/blueprints/WorldParams";

export interface ParallaxContainer extends Container {
  x_pscaling: number;
  y_pscaling: number;
}

export function set_blueprint_render(
  render_system: RenderSystem,
  game_instances: GameInstanceMap,
  blueprint_render_data: GameInstancesStore["blueprint_render"],
) {
  if (
    render_system.current_main_instance.instance_id &&
    render_system.current_main_instance.world_id
  ) {
    const worlds =
      game_instances[render_system.current_main_instance.instance_id];
    const game_instance = worlds[render_system.current_main_instance.world_id];
    if (
      game_instance.renderer.main_container &&
      blueprint_render_data &&
      blueprint_render_data.render_graph_data
    ) {
      blueprint_render_data.render_graph_data.render_root.container.alpha = 0.5;
      game_instance.renderer.blueprint_container.removeChildren();
      game_instance.renderer.blueprint_container.addChild(
        blueprint_render_data.render_graph_data.render_root.container,
      );
    }
  }
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
    blueprint_render_data: null,
    current_main_instance: {},
    global_mouse_position,
    stage: mainContainer,
  };
}

export const create_dummy_pic = async (color: string): Promise<ImageBitmap> => {
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
  world_params: WorldParams,
): InstanceRendering => {
  const main_container = new Container();
  const main_container_wrapper = new Container();
  const layer_map = createLayerMap();
  const blueprint_container = new Container();
  addLayerMapToContainer(main_container, layer_map);
  layer_map.ObjectsFront.addChild(blueprint_container);
  main_container_wrapper.addChild(main_container);
  const camera = create_camera();
  return {
    camera,
    layer_map,
    main_container,
    blueprint_container,
    main_container_wrapper,
    terrain_params: world_params.terrain_params,
  };
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
