import {
  CommunicationState,
  signal_channel_name,
  twitch_login_channel_name,
} from "../communication";
import { SignalToMedium } from "../communication/api/bindings/SignalToMedium";
import { twitch_service } from "../communication/api/twitch/twitch";
import {
  send_admin_event,
  send_system_event,
} from "../communication/setup_communication_system";
import { Config } from "../config";
import { is_admin } from "@/client/is_admin";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";
import { GuestToSystemEvent } from "@/client/communication/api/bindings/GuestToSystemEvent";

function send_twitch_login(
  communication_state: CommunicationState,
  message: { data: { auth_code?: string; access_token?: string } },
) {
  const event = {
    ProviderLoggedIn: {
      login_provider: "Twitch",
      auth_code: message.data.auth_code ? message.data.auth_code : null,
      access_token: message.data.access_token
        ? message.data.access_token
        : null,
    },
  };
  is_admin
    ? send_admin_event(event as AdminToSystemEvent, communication_state)
    : send_system_event(event as GuestToSystemEvent, communication_state);
}

export function login(communication_state: CommunicationState): Promise<void> {
  const twitch_login_channel = new BroadcastChannel(twitch_login_channel_name);
  const signal_channel = new BroadcastChannel(signal_channel_name);

  return new Promise((resolve, reject) => {
    try {
      twitch_login_channel.onmessage = async (message) => {
        try {
          send_twitch_login(communication_state, message);
        } catch (e) {
          console.error(e);
          reject(e);
        }
      };
      signal_channel.onmessage = async (message) => {
        const signal = message.data as SignalToMedium;
        if (signal === "LoginSuccess") {
          resolve();
        }

        if (signal === "LoginFailed") {
          reject();
        }

        twitch_login_channel.close();
        signal_channel.close();
      };

      if (twitch_service.authToken) {
        send_twitch_login(communication_state, {
          data: { access_token: twitch_service.authToken },
        });
      } else {
        window.open(
          `https://id.twitch.tv/oauth2/authorize?response_type=code&client_id=uchpfk924k24ozzra8f6i7bpthn33r&redirect_uri=${Config.getTwitchAuthRedirect()}`,
          "",
          "width=500,height=500",
        );
      }
    } catch (e) {
      console.error(e);
      reject(e);
    }
  });
}
