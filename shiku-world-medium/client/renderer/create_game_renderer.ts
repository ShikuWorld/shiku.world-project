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
  Renderer,
  Texture,
} from "pixi.js";
import { Config } from "../config";
import { create_camera } from "@/client/camera";
import { SimpleEventDispatcher } from "strongly-typed-events";
import {
  GameInstancesStore,
  render_key,
  RenderGraphData,
} from "@/editor/stores/game-instances";
import { GameInstanceMap } from "@/client/game-instance";
import { WorldParams } from "@/editor/blueprints/WorldParams";
import {
  get_stage_height,
  get_stage_width,
  set_stage_height,
  set_stage_width,
} from "@/client/config/config";
import { get_generic_game_node } from "@/editor/stores/resources";

export interface ParallaxContainer extends Container {
  x_pscaling: number;
  y_pscaling: number;
}

export function set_blueprint_render(
  render_system: RenderSystem,
  game_instances: GameInstanceMap,
  blueprint_render_data_old: GameInstancesStore["blueprint_render"] | undefined,
  blueprint_render_data_new: GameInstancesStore["blueprint_render"],
) {
  if (
    render_system.current_main_instance.instance_id &&
    render_system.current_main_instance.world_id
  ) {
    const worlds =
      game_instances[render_system.current_main_instance.instance_id];
    const game_instance = worlds[render_system.current_main_instance.world_id];
    if (game_instance) {
      if (
        blueprint_render_data_old &&
        blueprint_render_data_old.render_graph_data
      ) {
        blueprint_render_data_old.render_graph_data.entity_layer_manager.clear(
          game_instance.renderer.layer_map,
        );
      }
      if (
        blueprint_render_data_new &&
        blueprint_render_data_new.render_graph_data
      ) {
        blueprint_render_data_new.render_graph_data.entity_layer_manager.change_layer_transparency(
          0.5,
        );
        blueprint_render_data_new.render_graph_data.entity_layer_manager.attach_to_layer_map(
          game_instance.renderer.layer_map,
        );
        update_blueprint_render_positions(
          render_system,
          game_instances,
          blueprint_render_data_new.render_graph_data,
        );
      }
    }
  }
}

export function update_blueprint_render_positions(
  render_system: RenderSystem,
  game_instances: GameInstanceMap,
  render_graph_data: RenderGraphData,
) {
  if (
    render_system.current_main_instance.instance_id &&
    render_system.current_main_instance.world_id
  ) {
    const worlds =
      game_instances[render_system.current_main_instance.instance_id];
    const game_instance = worlds[render_system.current_main_instance.world_id];
    if (game_instance && render_graph_data) {
      for (const [id, render_node] of Object.entries(
        render_graph_data.entity_node_to_render_node_map,
      )) {
        const node = render_graph_data.entity_node_map[id];
        if (node) {
          render_graph_data.entity_layer_manager.update_container_position(
            render_key(get_generic_game_node(node)),
            render_node.container,
          );
        }
      }
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
  if (world_params.camera_ref) {
    console.log("Setting camera ref", world_params.camera_ref);
    camera.set_camera_ref(world_params.camera_ref);
  }

  return {
    camera,
    layer_map,
    main_container,
    blueprint_container,
    main_container_wrapper,
    terrain_params: world_params.terrain_params,
  };
};

export const setup_resizing = (renderer: RenderSystem) => {
  const canvas_wrapper = document.getElementById("canvas");

  if (!canvas_wrapper) {
    console.error("Could not find canvas wrapper!");
    return;
  }

  window.addEventListener("resize", () => {
    viewPortResize(
      canvas_wrapper.offsetWidth,
      canvas_wrapper.offsetHeight,
      renderer.renderer,
    );
  });
  viewPortResize(
    canvas_wrapper.offsetWidth,
    canvas_wrapper.offsetHeight,
    renderer.renderer,
  );

  const twitch_chat = document.getElementById("twitch-chat") || {
    className: "",
    clientWidth: 0,
    style: { width: "" },
  };
  const toggle_chat_element = document.getElementById("toggle-chat");
  if (toggle_chat_element && twitch_chat) {
    twitch_chat.className = "";
    toggle_chat_element.addEventListener("click", (e) => {
      const toggle = e.target as HTMLElement;
      if (toggle.innerHTML === "»") {
        twitch_chat.style.width = "0px";
        toggle.innerHTML = "«";
      } else {
        twitch_chat.style.width = "";
        toggle.innerHTML = "»";
      }
      viewPortResize(
        canvas_wrapper.offsetWidth,
        canvas_wrapper.offsetHeight,
        renderer.renderer,
      );
    });
  }
};

export const viewPortResize = (
  width: number,
  height: number,
  renderer: Renderer,
) => {
  renderer.canvas.style.width = `${width}px`;
  renderer.canvas.style.height = `${height}px`;

  set_stage_width(width);
  set_stage_height(height);

  renderer.resize(get_stage_width(), get_stage_height());
};
