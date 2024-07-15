import { create_game_renderer } from "./renderer/create_game_renderer";
import {
  check_for_connection_ready,
  send_admin_event,
  send_module_event,
  send_ticket,
  setup_communication_system,
} from "./communication/setup_communication_system";
import { MenuSystem } from "./menu";
import { create_guest_input } from "./input/create_guest_input";
import { render } from "./renderer/render";
import { match, P } from "ts-pattern";
import { createToast } from "./renderer/toast";
import { Button, create_guest_input_event } from "./input";
import { signal_channel_name } from "./communication";
import { initialize_input_plugins, update_input_plugins } from "./plugins";
import { setup_button_feedback } from "./button-feedback";
import { setup_medium_api } from "./api";
import {
  create_new_game_instance,
  GameInstanceMap,
} from "@/client/game-instance";
import { ResourceManagerMap } from "@/client/resources";
import { create_resource_manager } from "@/client/resources/create_resource_manager";
import { is_admin } from "@/client/is_admin";
import { handle_editor_event } from "@/client/handle-editor-event";
import { init_grid, toggle_grid } from "@/client/renderer/grid";

export async function start_medium() {
  const signal_broadcast_channel = new BroadcastChannel(signal_channel_name);

  const GUEST_SINGLE_WORLD_ID = "default";
  const canvas = document.getElementById("canvas");
  const door = document.getElementById("door");
  const render_system = await create_game_renderer();
  const menu_system = new MenuSystem();
  const communication_system = setup_communication_system(menu_system);
  const guest_input = create_guest_input();
  const button_feedback_update = setup_button_feedback();
  const instances: GameInstanceMap = {};
  const resource_manager_map: ResourceManagerMap = {};
  let current_active_instance_id: string | null = null;
  let current_active_module_id: string | null = null;

  function lazy_get_resource_manager(module_id: string) {
    if (!resource_manager_map[module_id]) {
      resource_manager_map[module_id] = create_resource_manager(
        render_system,
        module_id,
      );
      resource_manager_map[module_id].resource_bundle_complete.sub(() => {
        if (!is_admin) {
          send_module_event("GameSetupDone", communication_system);
        }
      });
    }
    return resource_manager_map[module_id];
  }

  initialize_input_plugins(guest_input);
  setup_medium_api(
    communication_system,
    instances,
    resource_manager_map,
    render_system,
  );

  /*renderer.onStageResize.sub((_resize) => {
        // TODO: Resize
        entity_manager.iterate_entities((e) => {
          if (e.layer_name === "Menu" && e.isometry.x < 0) {
            e.wrapper.x =
              resize.stage_width +
              Math.round(e.isometry.x * Config.get_simulation_scale());
          }
        });
      });*/

  if (door && canvas) {
    door.addEventListener("click", () => {
      canvas.className = "canvas--active";
    });
  }

  const open_menu = document.querySelector("#open-menu span");

  open_menu?.addEventListener("click", () => {
    menu_system.toggle(`${current_active_instance_id}Menu`);
  });

  function main_loop() {
    window.requestAnimationFrame(main_loop);

    render(render_system);

    if (door && guest_input.button_pressed_map[Button.Exit] && canvas) {
      canvas.className = "canvas--hidden";
    }

    update_input_plugins();
    button_feedback_update(guest_input);

    for (const communication_event of communication_system.inbox) {
      match(communication_event)
        .with("AlreadyConnected", () => {})
        .with({ EditorEvent: P.select() }, handle_editor_event)
        .with(
          { ConnectionReady: P.select() },
          ([_session_id, should_login]) => {
            console.log("Connection ready", should_login);
            if (should_login) {
              menu_system.activate(MenuSystem.static_menus.LoginMenu);
            } else if (is_admin) {
              window.medium_gui.editor.show_editor();
            }
          },
        )
        .with({ ResourceEvent: P.select() }, ([module_id, resource_event]) => {
          lazy_get_resource_manager(module_id).handle_resource_event(
            resource_event,
          );
        })
        .with(
          { PrepareGame: P.select() },
          async ([
            module_id,
            instance_id,
            w_id,
            resource_bundle,
            terrain_params,
            parralax_map,
            tilesets,
            gid_map,
            char_anim_to_tileset_map,
          ]) => {
            const resource_manager = lazy_get_resource_manager(module_id);
            console.log("setting gid_map", gid_map, module_id);
            resource_manager.gid_map = gid_map;
            resource_manager.tilesets = tilesets;
            resource_manager.set_tileset_map(tilesets);
            resource_manager.character_animation_to_tileset_map =
              char_anim_to_tileset_map;

            await resource_manager.load_resource_bundle(
              module_id,
              instance_id,
              resource_bundle,
              true,
            );

            if (!instances[instance_id]) {
              instances[instance_id] = {};
            }
            const world_id = w_id ? w_id : GUEST_SINGLE_WORLD_ID;
            instances[instance_id][world_id] = create_new_game_instance(
              instance_id,
              module_id,
              world_id,
              terrain_params,
            );
            current_active_instance_id = instance_id;
            current_active_module_id = module_id;
            if (world_id === GUEST_SINGLE_WORLD_ID) {
              render_system.stage.addChild(
                instances[instance_id][world_id].renderer
                  .main_container_wrapper,
              );
            }
            for (const [layer_kind, x, y] of parralax_map) {
              instances[instance_id][world_id].renderer.layer_map[
                layer_kind
              ].x_pscaling = x;
              instances[instance_id][world_id].renderer.layer_map[
                layer_kind
              ].y_pscaling = y;
            }
            if (is_admin) {
              init_grid(
                render_system,
                instances[instance_id][world_id].renderer,
              );
              toggle_grid(instances[instance_id][world_id].renderer);
              const guaranteed_world_id_as_admin = w_id!;
              send_admin_event(
                {
                  WorldInitialized: [
                    module_id,
                    instance_id,
                    guaranteed_world_id_as_admin,
                  ],
                },
                communication_system,
              );
            }
          },
        )
        .with({ UnloadGame: P.select() }, ([_, instance_id, w_id]) => {
          const world_id = w_id ? w_id : GUEST_SINGLE_WORLD_ID;
          if (instances[instance_id] && instances[instance_id][world_id]) {
            if (world_id === GUEST_SINGLE_WORLD_ID) {
              render_system.stage.removeChild(
                instances[instance_id][world_id].renderer
                  .main_container_wrapper,
              );
            }
            if (
              render_system.current_main_instance.instance_id === instance_id &&
              render_system.current_main_instance.world_id === world_id
            ) {
              render_system.stage.removeChild(
                instances[instance_id][world_id].renderer
                  .main_container_wrapper,
              );
            }
            instances[instance_id][world_id].destroy();
            delete instances[instance_id][world_id];
            if (Object.keys(instances[instance_id]).length === 0) {
              delete instances[instance_id];
            }
            window.medium_gui.game_instances.remove_game_instance(
              instance_id,
              world_id,
            );
          }
        })
        .with(
          { GameSystemEvent: P.select() },
          ([module_id, instance_id, w_id, game_system_event]) => {
            const world_id = w_id ? w_id : GUEST_SINGLE_WORLD_ID;
            if (instances[instance_id] && instances[instance_id][world_id]) {
              instances[instance_id][world_id].handle_game_system_event(
                game_system_event,
                menu_system,
                lazy_get_resource_manager(module_id),
              );
            }
          },
        )
        .with({ Signal: P.select() }, (signal_to_guest) => {
          if (signal_to_guest === "LoginSuccess") {
            menu_system.deactivate(MenuSystem.static_menus.LoginMenu);
            if (is_admin) {
              window.medium_gui.editor.show_editor();
            }
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

    for (const instances_per_world of Object.values(instances)) {
      for (const instance of Object.values(instances_per_world)) {
        instance.update();
      }
    }

    if (
      guest_input.is_dirty &&
      current_active_instance_id !== null &&
      current_active_module_id !== null
    ) {
      if (is_admin) {
        send_admin_event(
          {
            ControlInput: [
              current_active_module_id,
              current_active_instance_id,
              create_guest_input_event(guest_input),
            ],
          },
          communication_system,
        );
      } else {
        console.log("Sending control input guest");
        send_module_event(
          {
            ControlInput: create_guest_input_event(guest_input),
          },
          communication_system,
        );
      }
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

      check_for_connection_ready(menu_system, communication_system);
      if (communication_system.is_connection_ready) {
        window.requestAnimationFrame(main_loop);
        if (canvas) {
          canvas.className = "canvas--active";
        }
        clearInterval(interval_handle);
        return;
      }

      send_ticket(
        {
          session_id,
          admin_login: is_admin,
        },
        communication_system,
      );
    }
  }, 100);
}
