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
import { loginMenuConfig } from "@/client/login-menu";
import {
  create_new_game_instance,
  GameInstanceMap,
} from "@/client/game-instance";
import { ResourceManagerMap } from "@/client/resources";
import { create_resource_manager } from "@/client/resources/create_resource_manager";
import { is_admin } from "@/client/is_admin";
import { handle_editor_event } from "@/client/handle-editor-event";

export function start_medium() {
  const signal_broadcast_channel = new BroadcastChannel(signal_channel_name);

  const GUEST_SINGLE_WORLD_ID = "default";
  const canvas = document.getElementById("canvas");
  const door = document.getElementById("door");
  const render_system = create_game_renderer();
  const communication_system = setup_communication_system();
  const menu_system = new MenuSystem();
  const guest_input = create_guest_input();
  const button_feedback_update = setup_button_feedback();
  const instances: GameInstanceMap = {};
  const resource_manager_map: ResourceManagerMap = {};
  const current_active_instance: string | null = null;

  function lazy_get_resource_manager(module_id: string) {
    if (!resource_manager_map[module_id]) {
      resource_manager_map[module_id] = create_resource_manager();
      resource_manager_map[module_id].resource_bundle_complete.sub(
        ({ module_id }) => {
          if (is_admin) {
            send_admin_event(
              { InitialResourcesLoaded: module_id },
              communication_system,
            );
          } else {
            send_module_event("GameSetupDone", communication_system);
          }
        },
      );
    }

    return resource_manager_map[module_id];
  }

  initialize_input_plugins(guest_input);
  setup_medium_api(communication_system, instances, render_system);

  menu_system.create_menu_from_config(loginMenuConfig, "login-menu");

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
    menu_system.toggle(`${current_active_instance}Menu`);
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
            if (should_login) {
              menu_system.activate("login-menu");
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
          ([
            module_id,
            instance_id,
            w_id,
            resource_bundle,
            tilesets,
            gid_map,
          ]) => {
            const resource_manager = lazy_get_resource_manager(module_id);
            resource_manager.gid_map = gid_map;
            resource_manager.tilesets = tilesets;
            if (resource_manager_map[module_id]) {
              send_admin_event(
                { InitialResourcesLoaded: module_id },
                communication_system,
              );
            } else {
              resource_manager.load_resource_bundle(
                module_id,
                instance_id,
                resource_bundle,
              );
            }

            if (!instances[instance_id]) {
              instances[instance_id] = {};
            }
            const world_id = w_id ? w_id : GUEST_SINGLE_WORLD_ID;
            instances[instance_id][world_id] = create_new_game_instance(
              instance_id,
              module_id,
              world_id,
            );
            if (world_id === GUEST_SINGLE_WORLD_ID) {
              render_system.stage.addChild(
                instances[instance_id][world_id].renderer.mainContainerWrapper,
              );
            }
            window.medium_gui.editor.set_game_instances(instances);
          },
        )
        .with({ UnloadGame: P.select() }, ([_, instance_id, w_id]) => {
          const world_id = w_id ? w_id : GUEST_SINGLE_WORLD_ID;
          console.log("wait not even?", instance_id, world_id, instances);
          if (instances[instance_id] && instances[instance_id][world_id]) {
            if (world_id === GUEST_SINGLE_WORLD_ID) {
              render_system.stage.removeChild(
                instances[instance_id][world_id].renderer.mainContainerWrapper,
              );
            }
            console.log(
              render_system.current_main_instance,
              instance_id,
              world_id,
            );
            if (
              render_system.current_main_instance.instance_id === instance_id &&
              render_system.current_main_instance.world_id === world_id
            ) {
              console.log("removing?");
              render_system.stage.removeChild(
                instances[instance_id][world_id].renderer.mainContainerWrapper,
              );
            }
            instances[instance_id][world_id].destroy();
            delete instances[instance_id][world_id];
            if (Object.keys(instances[instance_id]).length === 0) {
              delete instances[instance_id];
            }
            window.medium_gui.editor.set_game_instances(instances);
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
        .with(
          { PositionEvent: P.select() },
          ([_module_id, instance_id, w_id, position_event]) => {
            const world_id = w_id ? w_id : "default";
            if (instances[instance_id] && instances[instance_id][world_id]) {
              instances[instance_id][world_id].handle_position_update(
                position_event,
              );
            }
          },
        )
        .with({ Signal: P.select() }, (signal_to_guest) => {
          if (signal_to_guest === "LoginSuccess") {
            menu_system.deactivate("login-menu");
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

    if (guest_input.is_dirty && current_active_instance !== null) {
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
          admin_login: is_admin,
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
