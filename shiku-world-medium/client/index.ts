import { twitch_login_channel_name } from "./communication";
import { start_medium } from "./game";
import { twitch_service } from "./communication/api/twitch/twitch";
import { setup_plugin_system } from "./plugins";

setup_plugin_system();

const twitch_broadcast_channel = new BroadcastChannel(
  twitch_login_channel_name,
);

window.onload = function () {
  const params = new URLSearchParams(window.location.search);
  if (params.has("code")) {
    twitch_broadcast_channel.postMessage({ auth_code: params.get("code") });
    window.close();
    return;
  }

  const door = document.getElementById("door");
  const canvas = document.getElementById("canvas");
  if (!canvas) {
    console.error("No canvas?!");
    return;
  }

  if (!door) {
    canvas.className = "canvas--loading";
    start_medium();
    return;
  }

  if (twitch_service.canIdentifyUser) {
    door.className = "door--open";
  } else {
    twitch_service.onAuth.subscribe(() => {
      if (twitch_service.canIdentifyUser) {
        door.className = "door--open";
      }
    });
  }

  const door_enter_click_handler = () => {
    if (twitch_service.canIdentifyUser) {
      canvas.className = "canvas--loading";
      door.removeEventListener("click", door_enter_click_handler);
      start_medium();
    } else {
      twitch_service.requestIdShare();
    }
  };

  door.addEventListener("click", door_enter_click_handler);
};
export { RenderGraph } from "@/client/render-graph";
export { Node } from "@/client/render-graph";
