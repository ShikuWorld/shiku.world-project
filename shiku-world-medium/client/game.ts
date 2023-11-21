import { create_game_renderer } from "./renderer/create_game_renderer";
import {
  check_for_connection_ready,
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
import { GameInstance, create_new_game_instance } from "@/client/game-instance";
import { ResourceManager } from "@/client/resources";
import { create_resource_manager } from "@/client/resources/create_resource_manager";
import { is_admin } from "@/client/is_admin";

export function start_medium() {
  const signal_broadcast_channel = new BroadcastChannel(signal_channel_name);

  const canvas = document.getElementById("canvas");
  const door = document.getElementById("door");
  const render_system = create_game_renderer();
  const communication_system = setup_communication_system();
  const menu_system = new MenuSystem();
  const guest_input = create_guest_input();
  const button_feedback_update = setup_button_feedback();
  const instances: { [instance_id: string]: GameInstance } = {};
  const resource_manager_map: { [module_name: string]: ResourceManager } = {};
  let current_active_instance: string | null = null;
  function lazy_get_resource_manager(module_name: string) {
    if (!resource_manager_map[module_name]) {
      resource_manager_map[module_name] = create_resource_manager();
      resource_manager_map[module_name].resource_bundle_complete.sub(() => {
        send_module_event("GameSetupDone", communication_system);
      });
    }

    return resource_manager_map[module_name];
  }

  initialize_input_plugins(guest_input);
  setup_medium_api(communication_system);

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
        .with({ EditorEvent: P.select() }, (e) => {
          match(e)
            .with({ MainDoorStatus: P.select() }, (status) => {
              window.medium_gui.editor.set_main_door_status(status);
            })
            .with({ Modules: P.select() }, (modules) => {
              window.medium_gui.editor.set_modules(modules);
            })
            .with({ CreatedModule: P.select() }, (d) => {
              window.medium_gui.editor.create_module(d[1]);
            })
            .with({ UpdatedModule: P.select() }, (d) => {
              window.medium_gui.editor.update_module(d[1]);
            })
            .with({ DeletedModule: P.select() }, (d) => {
              window.medium_gui.editor.delete_module(d);
            })
            .with({ CreatedTileset: P.select() }, (d) => {
              window.medium_gui.editor.set_tileset(d);
            })
            .with({ DirectoryInfo: P.select() }, (d) => {
              window.medium_gui.editor.set_current_file_browser_result(d);
            })
            .with({ UpdatedTileset: P.select() }, (d) => {
              window.medium_gui.editor.set_tileset(d);
            })
            .with({ DeletedTileset: P.select() }, (d) => {
              window.medium_gui.editor.delete_tileset(d);
            })
            .with({ UpdatedConductor: P.select() }, (d) => {
              window.medium_gui.editor.set_conductor(d);
            })
            .exhaustive();
        })
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
        .with(
          { ResourceEvent: P.select() },
          ([module_name, resource_event]) => {
            lazy_get_resource_manager(module_name).handle_resource_event(
              resource_event,
            );
          },
        )
        .with(
          { PrepareGame: P.select() },
          ([module_name, instance_id, resource_bundle, is_main_instance]) => {
            lazy_get_resource_manager(module_name).load_resource_bundle(
              module_name,
              instance_id,
              resource_bundle,
            );
            instances[instance_id] = create_new_game_instance(
              instance_id,
              module_name,
              render_system,
            );
            if (is_main_instance) {
              current_active_instance = instance_id;
              window.medium_gui.editor.set_current_main_instance_id(
                instance_id,
              );
            }
          },
        )
        .with(
          { GameSystemEvent: P.select() },
          ([module_name, instance_id, game_system_event]) => {
            instances[instance_id].handle_game_system_event(
              game_system_event,
              menu_system,
              lazy_get_resource_manager(module_name),
            );
          },
        )
        .with(
          { PositionEvent: P.select() },
          ([_moudule_name, instance_id, position_event]) => {
            if (instances[instance_id]) {
              instances[instance_id].handle_position_update(position_event);
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

    for (const instance of Object.values(instances)) {
      instance.update();
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
