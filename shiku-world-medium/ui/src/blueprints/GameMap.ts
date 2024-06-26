// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Chunk } from "./Chunk";
import type { LayerKind } from "./LayerKind";

export interface GameMap { module_id: string, world_id: string, name: string, resource_path: string, chunk_size: number, tile_width: number, tile_height: number, main_scene: string, terrain: Record<LayerKind, Record<number, Chunk>>, layer_parallax: Record<LayerKind, [number, number]>, }