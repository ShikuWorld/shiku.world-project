import { AnimatedSprite, Container, Sprite } from "pixi.js-legacy";

import { LayerName } from "../communication/api/bindings/LayerName";
import { TerrainChunk } from "../communication/api/bindings/TerrainChunk";
import { ResourceManager } from "../resources";
import { InstanceRendering } from "../renderer";

export function create_terrain_manager(): TerrainManager {
  return new TerrainManager();
}

type Chunk = {
  [chunk_key: string]: {
    x: number;
    y: number;
    container: Container;
  };
};

export class TerrainManager {
  private _chunk_map: Map<LayerName, Chunk>;

  constructor() {
    this._chunk_map = new Map<LayerName, Chunk>();
  }

  show_grid(_size: [number, number], _offset: [number, number]) {}

  hide_grid() {}

  remove_all_chunks_for_module(renderer: InstanceRendering) {
    for (const [layer, layer_chunks] of this._chunk_map.entries()) {
      for (const chunk of Object.values(layer_chunks)) {
        renderer.layerMap[layer].removeChild(chunk.container);
      }
    }
  }

  add_chunk(
    resource_manager: ResourceManager,
    renderer: InstanceRendering,
    tile_size: number,
    chunk: TerrainChunk,
  ) {
    if (!this._chunk_map.has(chunk.layer)) {
      this._chunk_map.set(chunk.layer, {});
    }

    const chunk_map = this._chunk_map.get(chunk.layer) as Chunk;
    chunk_map[`${chunk.x}-${chunk.y}`] = {
      y: 0,
      x: 0,
      container: new Container(),
    };

    const chunk_map_entry = chunk_map[`${chunk.x}-${chunk.y}`];
    chunk_map_entry.x = chunk.x * tile_size;
    chunk_map_entry.y = chunk.y * tile_size;

    /*const graphics = new Graphics();

    graphics.lineStyle(1, 0xff0000);
    graphics.drawRect(
      chunk_map_entry.x,
      chunk_map_entry.y,
      chunk.width * chunk.width,
      chunk.height * chunk.height
    );

    renderer.debugContainer.addChild(graphics);*/

    renderer.layerMap[chunk.layer].addChild(chunk_map_entry.container);

    for (const [y, row] of chunk.tile_ids.entries()) {
      for (const [x, gid] of row.entries()) {
        if (gid === 0) {
          continue;
        }

        const graphics = resource_manager.get_graphics_data_by_gid(
          gid,
          renderer,
        );

        let sprite: Sprite;

        if (graphics.frame_objects.length > 0) {
          const animated_sprite = new AnimatedSprite(graphics.frame_objects);

          animated_sprite.play();
          sprite = animated_sprite;
        } else {
          sprite = new Sprite(graphics.textures[0]);
        }

        sprite.anchor.set(0, 1);
        sprite.x = (chunk.x + x) * tile_size;
        sprite.y = (chunk.y + y) * tile_size;
        sprite.rotation = 0;

        chunk_map_entry.container.addChild(sprite);
      }
    }
  }
}
