// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CharacterAnimation } from "../blueprints/CharacterAnimation";
import type { Conductor } from "../blueprints/Conductor";
import type { Entity } from "../blueprints/Entity";
import type { EntityUpdate } from "../blueprints/EntityUpdate";
import type { GameMap } from "../blueprints/GameMap";
import type { GameNodeKind } from "../blueprints/GameNodeKind";
import type { GuestInput } from "./GuestInput";
import type { MapUpdate } from "../blueprints/MapUpdate";
import type { ModuleUpdate } from "../blueprints/ModuleUpdate";
import type { ProviderLoggedIn } from "./ProviderLoggedIn";
import type { Scene } from "../blueprints/Scene";
import type { SceneNodeUpdate } from "./SceneNodeUpdate";
import type { Script } from "../blueprints/Script";
import type { Tileset } from "../blueprints/Tileset";
import type { TilesetUpdate } from "./TilesetUpdate";

export type AdminToSystemEvent = { ProviderLoggedIn: ProviderLoggedIn } | { UpdateConductor: Conductor } | { BrowseFolder: string } | { OpenInstance: string } | { StartInspectingWorld: [string, string, string] } | { StopInspectingWorld: [string, string, string] } | { ControlInput: [string, string, GuestInput] } | { WorldInitialized: [string, string, string] } | { UpdateModule: [string, ModuleUpdate] } | { CreateModule: string } | { GetResource: string } | { CreateTileset: [string, Tileset] } | { SetTileset: Tileset } | { UpdateTileset: [string, TilesetUpdate] } | { DeleteTileset: Tileset } | { CreateScene: [string, Scene] } | { UpdateSceneNode: SceneNodeUpdate } | { UpdateInstancedNode: [string, string, string, EntityUpdate] } | { ResetGameWorld: [string, string, string] } | { OverwriteSceneRoot: [string, GameNodeKind] } | { RemoveInstanceNode: [string, string, string, Entity] } | { AddNodeToInstanceNode: [string, string, string, Entity, GameNodeKind] } | { DeleteScene: Scene } | { CreateMap: [string, GameMap] } | { UpdateMap: MapUpdate } | { DeleteMap: [string, GameMap] } | { CreateScript: [string, Script] } | { UpdateScript: Script } | { DeleteScript: Script } | { CreateCharacterAnimation: [string, CharacterAnimation] } | { UpdateCharacterAnimation: CharacterAnimation } | { DeleteCharacterAnimation: CharacterAnimation } | { DeleteModule: string } | { SetMainDoorStatus: boolean } | { SetBackDoorStatus: boolean } | "LoadEditorData" | "Ping";