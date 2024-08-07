// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Entity } from "./Entity";
import type { GameNodeKind } from "./GameNodeKind";

export interface GameNode<T> { id: string, name: string, entity_id: Entity | null, data: T, script: string | null, tags: Array<string>, instance_resource_path: string | null, children: Array<GameNodeKind>, }