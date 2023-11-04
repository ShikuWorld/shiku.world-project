import { CommunicationEvent } from "./api/bindings/Events";
import { CommunicationState } from "./index";
import { Ticket } from "./api/bindings/Ticket";
import { GuestToModuleEvent } from "./api/bindings/GuestToModuleEvent";
import { Config } from "../config";
import { GuestTo } from "@/client/communication/api/bindings/GuestTo";
import { GuestToSystemEvent } from "@/client/communication/api/bindings/GuestToSystemEvent";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { is_admin } from "@/client/is_admin";

export function setup_communication_system(): CommunicationState {
  const ws_connection = new WebSocket(Config.getWsSocketUrl());
  const communication_state: CommunicationState = {
    is_connection_open: false,
    is_connection_ready: false,
    inbox: [],
    ws_connection,
  };

  ws_connection.onopen = () => {
    communication_state.is_connection_open = true;
  };
  ws_connection.onclose = (close_event) => {
    const message = document.createElement("div");
    if (close_event.reason === "Logged in elsewhere") {
      message.innerHTML =
        "You seem to have logged in somewhere else, please login again if you want to use this device.";
    } else {
      message.innerHTML = "Connection to server closed, please try and reload.";
    }
    communication_state.is_connection_open = false;
    document.body.prepend(message);
    document.querySelector("canvas")?.remove();
  };

  ws_connection.onmessage = (message: MessageEvent) => {
    try {
      communication_state.inbox.push(
        JSON.parse(message.data) as CommunicationEvent,
      );
    } catch (e) {
      console.error(e);
    }
  };

  ws_connection.onerror = (event) => {
    console.error(event);
    communication_state.is_connection_open = false;
    const message = document.createElement("div");
    message.innerHTML =
      "Connection to server encountered error, please try and reload.";
    document.body.prepend(message);
  };

  return communication_state;
}

let last_message_send = Date.now();

export function check_for_connection_ready(
  communication_state: CommunicationState,
) {
  for (const communication of communication_state.inbox) {
    if (communication == "AlreadyConnected") {
      communication_state.ws_connection.close(1000);
      const message = document.createElement("div");
      message.innerHTML =
        "You are already connected somewhere. Maybe check your browser tabs?";
      document.body.prepend(message);
      continue;
    }
    if ("ConnectionReady" in communication) {
      communication_state.is_connection_ready = true;
      try {
        sessionStorage.setItem("session_id", communication.ConnectionReady[0]);
      } catch (e) {
        console.error(
          "Seems like you block local storage or something, you'll have to login on every reload.",
        );
      }

      setInterval(() => {
        if (Date.now() - last_message_send > 10000) {
          is_admin
            ? send_admin_event("Ping", communication_state)
            : send_system_event("Ping", communication_state);
        }
      }, 10000);
    }
  }
}

export function send_system_event(
  input: GuestToSystemEvent,
  communication_state: CommunicationState,
) {
  send_event<GuestTo>({ GuestToSystemEvent: input }, communication_state);
}

export function send_admin_event(
  input: AdminToSystemEvent,
  communication_state: CommunicationState,
) {
  send_event<AdminToSystemEvent>(input, communication_state);
}

export function send_module_event(
  input: GuestToModuleEvent,
  communication_state: CommunicationState,
) {
  send_event<GuestTo>({ GuestToModuleEvent: input }, communication_state);
}

export function send_event<T>(
  input: T,
  communication_state: CommunicationState,
) {
  if (communication_state.is_connection_open) {
    communication_state.ws_connection.send(JSON.stringify(input));
    last_message_send = Date.now();
  }
}

export function send_ticket(
  ticket: Ticket,
  communication_state: CommunicationState,
) {
  communication_state.ws_connection.send(JSON.stringify(ticket));
}
