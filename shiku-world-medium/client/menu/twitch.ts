import {
  CommunicationState,
  signal_channel_name,
  twitch_login_channel_name,
} from "../communication";
import { SignalToGuest } from "../communication/api/bindings/SignalToGuest";
import { twitch_service } from "../communication/api/twitch/twitch";
import { send_system_event } from "../communication/setup_communication_system";
import { Config } from "../config";

function send_twitch_login(
  communication_state: CommunicationState,
  message: { data: { auth_code?: string; access_token?: string } },
) {
  send_system_event(
    {
      ProviderLoggedIn: {
        login_provider: "Twitch",
        auth_code: message.data.auth_code ? message.data.auth_code : null,
        access_token: message.data.access_token
          ? message.data.access_token
          : null,
      },
    },
    communication_state,
  );
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
        const signal = message.data as SignalToGuest;
        if (signal === "LoginSuccess") {
          resolve();
        }

        if (signal === "LoginFailed") {
          reject();
        }

        twitch_login_channel.close();
        signal_channel.close();
      };

      if (twitch_service.auth && twitch_service.auth.token) {
        send_twitch_login(communication_state, {
          data: { access_token: twitch_service.auth.token },
        });
      } else {
        window.open(
          `https://api.twitch.tv/kraken/oauth2/authorize?response_type=code&client_id=uchpfk924k24ozzra8f6i7bpthn33r&redirect_uri=${Config.getTwitchAuthRedirect()}`,
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
