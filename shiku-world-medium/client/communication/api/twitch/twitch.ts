import { SimpleEventDispatcher } from "strongly-typed-events";
import { getMainDoorStatusUrl } from "@/client/config/config";

class TwitchService {
  auth?: Twitch.ext.Authorized;
  onAuth: SimpleEventDispatcher<Twitch.ext.Authorized>;

  constructor() {
    this.onAuth = new SimpleEventDispatcher<Twitch.ext.Authorized>();
  }

  get canIdentifyUser() {
    return Twitch.ext.viewer && Twitch.ext.viewer.isLinked;
  }

  getConfigurationValue(key: string): string | undefined {
    if (!window.Twitch || !Twitch.ext.configuration.broadcaster) {
      return undefined;
    }
    try {
      return JSON.parse(Twitch.ext.configuration.broadcaster.content)?.[key];
    } catch (e) {
      return undefined;
    }
  }

  get wsSocketUrl(): string | undefined {
    return this.getConfigurationValue("websocketurl");
  }

  get mainDoorStatusUrl(): string | undefined {
    return this.getConfigurationValue("mainDoorStatusUrl");
  }

  get backDoorStatusUrl(): string | undefined {
    return this.getConfigurationValue("backDoorStatusUrl");
  }

  get resourceUrl(): string | undefined {
    return this.getConfigurationValue("resourceUrl");
  }

  get twitchAuthRedirect(): string | undefined {
    return this.getConfigurationValue("twitchAuthRedirect");
  }

  requestIdShare() {
    Twitch.ext.actions.requestIdShare();
  }
}

function setupTwitchService(): TwitchService {
  const twitch_service = new TwitchService();

  if (!window.Twitch) {
    return twitch_service;
  }

  const twitch = window.Twitch.ext;

  twitch.rig.log("Twitch setup");

  twitch.onContext(function (context) {
    twitch.rig.log(context && context.mode ? context.mode : "");
  });

  twitch.onAuthorized(function (auth) {
    if (Twitch.ext.viewer && Twitch.ext.viewer.isLinked) {
      twitch_service.auth = auth;
      twitch_service.onAuth.dispatch(auth);
    } else {
      Twitch.ext.actions.requestIdShare();
    }
  });

  return twitch_service;
}

export const twitch_service = setupTwitchService();
