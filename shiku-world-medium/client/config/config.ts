import { twitch_service } from "../communication/api/twitch/twitch";
import { environment } from "../environment";

export const get_simulation_scale = () => {
  return 100;
};

const BG_COLOR = 0x000000;
let camera_zoom = 1.0;
let stage_width = window.innerWidth;
let stage_height = window.outerHeight;

export const set_camera_zoom = (zoom: number) => {
  camera_zoom = zoom;
};

export const get_camera_zoom = () => {
  return camera_zoom;
};

export const set_stage_width = (width: number) => {
  stage_width = width;
};

export const set_stage_height = (height: number) => {
  stage_height = height;
};

export const get_stage_width = (): number => {
  return stage_width;
};

export const get_stage_height = (): number => {
  return stage_height;
};

export const get_bg_color = (): number => {
  return BG_COLOR;
};

export const get_enable_zoom = (): boolean => {
  return (
    new URL(window.location.href).searchParams.get("enable-zoom") === "true"
  );
};

export const getWsSocketUrl = () => {
  return twitch_service.wsSocketUrl || environment.wsSocketUrl;
};

export const get_resource_url = () => {
  return twitch_service.resourceUrl || environment.resourceUrl;
};

export const getTwitchAuthRedirect = () => {
  return twitch_service.twitchAuthRedirect || environment.twitchAuthRedirect;
};

export const getMainDoorStatusUrl = () => {
  return twitch_service.mainDoorStatusUrl || environment.mainDoorStatusUrl;
};

export const getBackDoorStatusUrl = () => {
  return twitch_service.backDoorStatusUrl || environment.backDoorStatusUrl;
};
