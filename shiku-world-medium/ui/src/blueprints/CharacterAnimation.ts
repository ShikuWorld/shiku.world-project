export type CharacterDirection = "Up" | "Down" | "Left" | "Right";
export interface CharacterAnimationFrame {
  duration_in_ms: number;
  gid_map: Record<CharacterDirection, number>;
}
export interface CharacterAnimationState {
  name: string;
  frames: Array<CharacterAnimationFrame>;
}
export interface CharacterAnimation {
  id: string;
  name: string;
  resource_path: string;
  tileset_resource: string;
  current_direction: CharacterDirection;
  current_state: number;
  current_gid_inside_tile: number;
  states: Record<number, CharacterAnimationState>;
  transitions: Record<number, Record<number, number>>;
}
