// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Image } from "./Image";
import type { Tile } from "./Tile";

export interface Tileset { name: string, resource_path: string, image: Image | null, tile_width: number, tile_height: number, tile_count: number, columns: number, tiles: Record<number, Tile>, }