import { login } from "../menu/twitch";
import { CommunicationState } from "../communication";

export const setup_medium_api = (communication_state: CommunicationState) => {
  window.medium = {
    twitch_login: login,
    communication_state: communication_state,
  };
};
