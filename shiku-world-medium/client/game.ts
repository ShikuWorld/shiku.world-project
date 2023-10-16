import { setup_rendering_system } from "./renderer/setup_rendering_system";
import {
  check_for_connection_ready,
  send_module_event,
  send_ticket,
  setup_communication_system,
} from "./communication/setup_communication_system";
import { setup_resource_manager } from "./resources/setup_resource_manager";
import { MenuSystem, setup_automatic_menu_creation } from "./menu";
import { create_entity_manager } from "./entities";
import { create_guest_input } from "./input/create_guest_input";
import { create_camera, set_container_to_viewport_coordinate } from "./camera";
import { create_terrain_manager } from "./terrain";
import { render } from "./renderer/render";
import { match, P } from "ts-pattern";
import { MediumDataStorage } from "./communication/api/bindings/MediumDataStorage";
import { createToast } from "./renderer/toast";
import { Button, create_guest_input_event } from "./input";
import { signal_channel_name } from "./communication";
import {
  get_plugin,
  initialize_input_plugins,
  update_input_plugins,
} from "./plugins";
import { setup_button_feedback } from "./button-feedback";
import { new_shaker } from "./renderer/shaker-factory";
import { MousePluginType } from "../plugins/mouse-input";
import { setup_medium_api } from "./api";
import { LayerName } from "./communication/api/bindings/LayerName";
import { Config } from "./config";
import { loginMenuConfig } from "@/client/login-menu";

export function start_medium() {
  const signal_broadcast_channel = new BroadcastChannel(signal_channel_name);

  const canvas = document.getElementById("canvas");
  const door = document.getElementById("door");
  const renderer = setup_rendering_system();
  const communication_system = setup_communication_system();
  const resource_manager = setup_resource_manager();
  const menu_system = new MenuSystem();
  const entity_manager = create_entity_manager();
  const guest_input = create_guest_input();
  const camera = create_camera();
  const terrain_manager = create_terrain_manager();
  const button_feedback_update = setup_button_feedback();
  let temp_current_module: string;

  initialize_input_plugins(guest_input);
  setup_medium_api(communication_system);

  menu_system.create_menu_from_config(loginMenuConfig, "login-menu");

  renderer.onStageResize.sub((resize) => {
    entity_manager.iterate_entities((e) => {
      if (e.layer_name === "Menu" && e.isometry.x < 0) {
        e.wrapper.x =
          resize.stage_width +
          Math.round(e.isometry.x * Config.get_simulation_scale());
      }
    });
  });

  resource_manager.resources_complete.sub((event) => {
    send_module_event(
      {
        ResourcesLoaded: event.module_name,
      },
      communication_system,
    );
  });

  setup_automatic_menu_creation(resource_manager, menu_system);

  if (door && canvas) {
    door.addEventListener("click", () => {
      canvas.className = "canvas--active";
    });
  }

  const open_menu = document.querySelector("#open-menu span");

  open_menu?.addEventListener("click", () => {
    menu_system.toggle(`${temp_current_module}Menu`);
  });

  function main_loop() {
    window.requestAnimationFrame(main_loop);

    render(renderer);

    if (door && guest_input.button_pressed_map[Button.Exit] && canvas) {
      canvas.className = "canvas--hidden";
    }

    update_input_plugins();
    button_feedback_update(guest_input);

    for (const communication_event of communication_system.inbox) {
      match(communication_event)
        .with("AlreadyConnected", () => {})
        .with(
          { ConnectionReady: P.select() },
          ([_session_id, should_login]) => {
            if (should_login) {
              menu_system.activate("login-menu");
            }
          },
        )
        .with({ ResourceEvent: P.select() }, (resource_event) => {
          match(resource_event)
            .with({ LoadResource: P.select() }, (module_resources_map) => {
              for (const module_name in module_resources_map) {
                temp_current_module = module_name;
                for (const resource of module_resources_map[module_name]) {
                  resource_manager.add_resource_to_loading_queue(
                    module_name,
                    resource,
                  );
                }
                resource_manager.start_loading(module_name);
              }
            })
            .with({ UnLoadResource: P.select() }, (module_name_to_unload) =>
              resource_manager.unload_resources(module_name_to_unload),
            )
            .exhaustive();
        })
        .with({ GameSystemEvent: P.select() }, (game_system_event) => {
          match(game_system_event)
            .with({ SetCamera: P.select() }, (camera_props) => {
              camera.set_camera_ref(camera_props[0], camera_props[1]);
              camera.set_camera_settings(camera_props[2], renderer);
            })
            .with({ OpenMenu: P.select() }, (menuName) => {
              menu_system.activate(menuName);
            })
            .with({ CloseMenu: P.select() }, (menuName) => {
              menu_system.deactivate(menuName);
            })
            .with(
              { ShowEntities: P.select() },
              ([show_entities, moduleName]) => {
                for (const show_entity of show_entities.filter(
                  (s) => !s.parent_entity,
                )) {
                  entity_manager.add_entity(
                    moduleName,
                    show_entity,
                    renderer,
                    resource_manager,
                  );
                }

                for (const show_entity of show_entities.filter(
                  (s) => s.parent_entity,
                )) {
                  entity_manager.add_entity(
                    moduleName,
                    show_entity,
                    renderer,
                    resource_manager,
                  );
                }
              },
            )
            .with({ RemoveAllEntities: P.select() }, (moduleName) => {
              entity_manager.remove_all_entities_from_module(moduleName);
              terrain_manager.remove_all_chunks_for_module(
                resource_manager,
                renderer,
                moduleName,
              );
            })
            .with(
              { RemoveEntities: P.select() },
              ([remove_entities, moduleName]) => {
                for (const remove_entity of remove_entities) {
                  entity_manager.remove_entity(moduleName, remove_entity);
                }
              },
            )
            .with(
              { ChangeEntity: P.select() },
              ([update_entities, _moduleName]) => {
                console.log(update_entities);
              },
            )
            .with(
              { ShowTerrainChunks: P.select() },
              ([tile_size, chunks, module_name]) => {
                for (const chunk of chunks) {
                  terrain_manager.add_chunk(
                    resource_manager,
                    renderer,
                    module_name,
                    tile_size,
                    chunk,
                  );
                }
              },
            )
            .with({ UpdateDataStore: P.select() }, (store_update) => {
              try {
                const update = JSON.parse(store_update) as MediumDataStorage;
                window.medium_gui.current_module.set_data(update);
              } catch (e) {
                console.error("Could not parse store update!");
              }
            })
            .with({ SetMouseInputSchema: P.select() }, (mouse_mode) => {
              const mouse_plugin = get_plugin("MOUSE") as
                | MousePluginType
                | undefined;
              if (mouse_plugin) {
                mouse_plugin.plugin_options.mouse_mode = mouse_mode;
              }
            })
            .with(
              { ShowEffects: P.select() },
              ([show_effects, module_name]) => {
                for (const show_effect of show_effects) {
                  match(show_effect)
                    .with(
                      { SimpleImageEffect: P.select() },
                      (simple_image_effect) => {
                        entity_manager.add_simple_image_effect(
                          module_name,
                          simple_image_effect,
                          renderer,
                          resource_manager,
                        );
                      },
                    )
                    .with(
                      { ShakeScreenEffect: P.select() },
                      (shake_screen_effect) => {
                        new_shaker({
                          target: renderer.worldContainer,
                          isBidirectional: shake_screen_effect.is_bidirectional,
                          shakeCountMax: shake_screen_effect.shake_count_max,
                          shakeDelay: shake_screen_effect.shake_delay,
                          shakeAmount: shake_screen_effect.shake_amount,
                        }).shake();
                      },
                    )
                    .exhaustive();
                }
              },
            )
            .with({ SetParallax: P.select() }, (layer_parallax) => {
              for (const key in renderer.layerContainer) {
                const parallax_container =
                  renderer.layerContainer[key as LayerName];
                parallax_container.y_pscaling = 1.0;
                parallax_container.x_pscaling = 1.0;
              }
              for (const [layer_name, parallax] of layer_parallax) {
                if (layer_name === "Menu") {
                  continue;
                }
                renderer.layerContainer[layer_name].x_pscaling = parallax[0];
                renderer.layerContainer[layer_name].y_pscaling = parallax[1];
              }
            })
            .with(
              { UpdateEntities: P.select() },
              ([updated_entities, module_name]) => {
                for (const update_entity of updated_entities) {
                  entity_manager.update_entity(
                    module_name,
                    update_entity,
                    resource_manager,
                    renderer,
                  );
                }
              },
            )
            .exhaustive();
        })
        .with({ PositionEvent: P.select() }, (position_event) => {
          for (const position_update of position_event) {
            entity_manager.update_entity_position(
              // TODO: Module should be implicitly known as every module has its own webrtc connection for positional data
              temp_current_module,
              position_update[0],
              position_update[1],
              position_update[2],
              position_update[3],
            );
          }
        })
        .with({ Signal: P.select() }, (signal_to_guest) => {
          if (signal_to_guest === "LoginSuccess") {
            menu_system.deactivate("login-menu");
          }
          signal_broadcast_channel.postMessage(signal_to_guest);
        })
        .with({ Toast: P.select() }, ([alertLevel, message]) => {
          createToast(alertLevel, message);
        })
        .with({ ShowGlobalMessage: P.select() }, (global_message) => {
          const element = document.querySelector("#temp");
          const newThingy = document.createElement("div");
          newThingy.innerText = global_message;
          element?.appendChild(newThingy);
          setTimeout(() => {
            element?.removeChild(newThingy);
          }, 30000);
        })
        .exhaustive();
    }
    communication_system.inbox = [];

    camera.update_camera_position(entity_manager, renderer);

    for (const key in renderer.layerContainer) {
      const layerName = key as LayerName;
      if (layerName === "Menu") {
        continue;
      }
      const parallax_container = renderer.layerContainer[layerName];

      set_container_to_viewport_coordinate(
        camera.camera_isometry,
        parallax_container,
      );
    }

    if (guest_input.is_dirty) {
      send_module_event(
        {
          ControlInput: create_guest_input_event(guest_input),
        },
        communication_system,
      );
      guest_input.is_dirty = false;
    }
  }

  const interval_handle = setInterval(() => {
    if (communication_system.is_connection_open) {
      let session_id = null;

      try {
        session_id = sessionStorage.getItem("session_id");
      } catch (e) {
        console.error(
          "Seems like you block local storage or something, you'll have to login on every reload.",
        );
      }

      send_ticket(
        {
          session_id,
          admin_login: null,
        },
        communication_system,
      );
      check_for_connection_ready(communication_system);
      if (communication_system.is_connection_ready) {
        window.requestAnimationFrame(main_loop);
        if (canvas) {
          canvas.className = "canvas--active";
        }
        clearInterval(interval_handle);
      }
    }
  }, 1000);
}
