// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Collider } from "./Collider";
import type { KinematicCharacterControllerProps } from "./KinematicCharacterControllerProps";
import type { RenderKind } from "./RenderKind";
import type { RigidBodyType } from "./RigidBodyType";
import type { ScopeCacheValue } from "./ScopeCacheValue";
import type { Transform } from "./Transform";

export type EntityUpdateKind = { Transform: Transform } | { Name: string } | { Tags: Array<string> } | { InstancePath: string } | { ScriptPath: string | null } | { UpdateScriptScope: [string, ScopeCacheValue] } | { SetScriptScope: Record<string, ScopeCacheValue> } | { RigidBodyType: RigidBodyType } | { KinematicCharacterControllerProps: KinematicCharacterControllerProps } | { Collider: Collider } | { PositionRotation: [number, number, number] } | { RenderKind: RenderKind } | { AnimatedSpriteResource: string } | { SpriteTilesetResource: string } | { Gid: number };