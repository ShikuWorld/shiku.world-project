export const RENDER_SCALE = 32.0;

export type DoorStatusCheck =
  | { type: "open" }
  | { type: "lightsOn" }
  | { type: "lightsOut" }
  | { type: "urlNotConfigured" }
  | { type: "unknownError"; error: Error };
