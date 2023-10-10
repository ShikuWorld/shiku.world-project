import { CommunicationEvent } from "./api/bindings/Events";

export const twitch_login_channel_name = "twitch";
export const signal_channel_name = "signal";

export interface CommunicationState {
  ws_connection: WebSocket;
  inbox: CommunicationEvent[];
  is_connection_open: boolean;
  is_connection_ready: boolean;
}
