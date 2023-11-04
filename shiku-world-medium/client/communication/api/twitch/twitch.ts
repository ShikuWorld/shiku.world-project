import { SimpleEventDispatcher } from "strongly-typed-events";

class TwitchService {
  auth?: Twitch.ext.Authorized;
  onAuth: SimpleEventDispatcher<Twitch.ext.Authorized>;

  constructor() {
    this.onAuth = new SimpleEventDispatcher<Twitch.ext.Authorized>();
  }

  get canIdentifyUser() {
    return Twitch.ext.viewer && Twitch.ext.viewer.isLinked;
  }

  get wsSocketUrl(): string | undefined {
    if (!window.Twitch || !Twitch.ext.configuration.broadcaster) {
      return void 0;
    }
    try {
      return JSON.parse(Twitch.ext.configuration.broadcaster.content)
        ?.websocketurl;
    } catch (e) {
      return void 0;
    }
  }

  get resourceUrl() {
    if (!window.Twitch || !Twitch.ext.configuration.broadcaster) {
      return void 0;
    }
    try {
      return JSON.parse(Twitch.ext.configuration.broadcaster.content)
        ?.resourceUrl;
    } catch (e) {
      return void 0;
    }
  }

  get twitchAuthRedirect() {
    if (!window.Twitch || !Twitch.ext.configuration.broadcaster) {
      return void 0;
    }
    try {
      return JSON.parse(Twitch.ext.configuration.broadcaster.content)
        ?.twitchAuthRedirect;
    } catch (e) {
      return void 0;
    }
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
