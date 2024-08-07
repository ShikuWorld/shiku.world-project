// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CharacterAnimationState } from "./CharacterAnimationState";
import type { CharacterDirection } from "./CharacterDirection";

export interface CharacterAnimation { id: string, name: string, resource_path: string, tileset_resource: string, start_direction: CharacterDirection, start_state: number, states: Record<number, CharacterAnimationState>, }