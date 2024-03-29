// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CameraSettings } from "./CameraSettings";
import type { Chunk } from "../blueprints/Chunk";
import type { GameNodeKind } from "../blueprints/GameNodeKind";
import type { LayerKind } from "../blueprints/LayerKind";
import type { LayerName } from "./LayerName";
import type { MouseInputSchema } from "./MouseInputSchema";
import type { Scene } from "../blueprints/Scene";
import type { ShowEntity } from "./ShowEntity";

export type GameSystemToGuestEvent = { OpenMenu: string } | { CloseMenu: string } | { UpdateDataStore: string } | { ShowTerrain: Array<[LayerKind, Array<Chunk>]> } | { SetParallax: Array<[LayerName, [number, number]]> } | { ShowScene: Scene } | { UpdateSceneNodes: Array<GameNodeKind> } | { RemoveSceneNodes: Array<string> } | { SetMouseInputSchema: MouseInputSchema } | { ChangeEntity: Array<ShowEntity> } | { SetCamera: [string, CameraSettings] };