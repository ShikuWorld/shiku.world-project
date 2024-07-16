import { CommunicationEvent } from "./api/bindings/Events";
import { CommunicationState } from "./index";
import { Ticket } from "./api/bindings/Ticket";
import { GuestToModuleEvent } from "./api/bindings/GuestToModuleEvent";
import { Config } from "../config";
import { GuestTo } from "@/client/communication/api/bindings/GuestTo";
import { GuestToSystemEvent } from "@/client/communication/api/bindings/GuestToSystemEvent";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { is_admin } from "@/client/is_admin";
import { MenuSystem } from "@/client/menu";

export function setup_communication_system(
  menu_system: MenuSystem,
): CommunicationState {
  const ws_connection = new WebSocket(Config.getWsSocketUrl());
  const communication_state: CommunicationState = {
    is_connection_open: false,
    is_connection_ready: false,
    inbox: [],
    ws_connection,
  };
  reset_communication_system(
    communication_state,
    menu_system,
    undefined,
    ws_connection,
  );

  return communication_state;
}

export function reset_communication_system(
  communication_state: CommunicationState,
  menu_system: MenuSystem,
  on_open_callback?: () => void,
  websocket?: WebSocket,
) {
  const ws_connection = websocket
    ? websocket
    : new WebSocket(Config.getWsSocketUrl());
  const mainDoorStatusUrl = Config.getMainDoorStatusUrl();
  ws_connection.onopen = () => {
    communication_state.is_connection_open = true;
    if (on_open_callback) {
      on_open_callback();
    }
  };
  ws_connection.onclose = (close_event) => {
    window.medium.hide_loading_indicator();
    if (close_event.reason === "Logged in elsewhere") {
      menu_system.activate(MenuSystem.static_menus.ReconnectMenu, {
        connection_error: {
          type: "logged_in_elsewhere",
          message:
            "You seem to have logged in somewhere else, please login again if you want to use this device.",
          mainDoorStatusUrl,
        },
      });
    } else {
      menu_system.activate(MenuSystem.static_menus.ReconnectMenu, {
        connection_error: {
          type: "connection_closed",
          message:
            "Seems like the connection to the server was closed, please try to reconnect.",
          mainDoorStatusUrl,
        },
      });
    }
    communication_state.is_connection_open = false;
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
    communication_state.is_connection_open = false;
    window.medium.hide_loading_indicator();
    menu_system.activate(MenuSystem.static_menus.ReconnectMenu, {
      connection_error: {
        type: "connection_error",
        message:
          "Connection to server encountered an error, please try to reconnect.",
        mainDoorStatusUrl,
      },
    });
    console.error(event);
  };
  communication_state.is_connection_open = false;
  communication_state.is_connection_ready = false;
  communication_state.inbox = [];
  communication_state.ws_connection = ws_connection;
}

let last_message_send = Date.now();

export function check_for_connection_ready(
  menu_system: MenuSystem,
  communication_state: CommunicationState,
) {
  for (const communication of communication_state.inbox) {
    if (communication == "AlreadyConnected") {
      communication_state.ws_connection.close(1000);
      menu_system.activate(MenuSystem.static_menus.ReconnectMenu, {
        connection_error: {
          type: "already_connected",
          message:
            "You are already connected somewhere. Maybe check your browser tabs?",
        },
      });
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
