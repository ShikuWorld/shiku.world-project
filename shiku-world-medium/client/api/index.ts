import { login } from "../menu/twitch";
import { CommunicationState } from "../communication";
import { GameInstanceMap } from "@/client/game-instance";
import { RenderSystem } from "@/client/renderer";

export const setup_medium_api = (
  communication_state: CommunicationState,
  instances: GameInstanceMap,
  render_system: RenderSystem,
) => {
  window.medium = {
    twitch_login: login,
    communication_state: communication_state,
    swap_main_render_instance: (instance_id, world_id) => {
      if (instances[instance_id] && instances[instance_id][world_id]) {
        if (
          render_system.current_main_instance.instance_id &&
          render_system.current_main_instance.world_id
        ) {
          if (
            instances[render_system.current_main_instance.instance_id] &&
            instances[render_system.current_main_instance.instance_id][
              render_system.current_main_instance.world_id
            ]
          ) {
            render_system.stage.removeChild(
              instances[render_system.current_main_instance.instance_id][
                render_system.current_main_instance.world_id
              ].renderer.mainContainerWrapper,
            );
          }
        }
        render_system.stage.addChild(
          instances[instance_id][world_id].renderer.mainContainerWrapper,
        );
        render_system.current_main_instance = { instance_id, world_id };
      }
    },
  };
};
