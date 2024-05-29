import { Container } from "pixi.js";
import { ResourceManager } from "../resources";
import { InstanceRendering } from "../renderer";
import { TerrainParams } from "@/editor/blueprints/TerrainParams";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { Chunk } from "@/editor/blueprints/Chunk";

export function create_terrain_manager(
  terrain_params: TerrainParams,
): TerrainManager {
  return new TerrainManager(terrain_params);
}

export class TerrainManager {
  private _chunk_map: Map<
    LayerKind,
    Map<number, { container: Container; data: Chunk }>
  >;

  constructor(public terrain_params: TerrainParams) {
    this._chunk_map = new Map();
  }

  remove_all_chunks_for_module(renderer: InstanceRendering) {
    for (const [layer, layer_chunks] of this._chunk_map.entries()) {
      for (const chunk of Object.values(layer_chunks)) {
        renderer.layer_map[layer].removeChild(chunk.container);
      }
    }
  }

  add_chunk(
    resource_manager: ResourceManager,
    renderer: InstanceRendering,
    layer_kind: LayerKind,
    chunk: Chunk,
  ) {
    if (!this._chunk_map.has(layer_kind)) {
      this._chunk_map.set(layer_kind, new Map());
    }
    const chunk_map = this._chunk_map.get(layer_kind)!;
    const chunk_key = this._cantorPair(chunk.position[0], chunk.position[1]);
    if (!chunk_map.has(chunk_key)) {
      const chunk_map_entry = {
        container: new Container(),
        data: chunk,
      };
      chunk_map_entry.container.x =
        chunk.position[0] *
        this.terrain_params.chunk_size *
        this.terrain_params.tile_width;
      chunk_map_entry.container.y =
        chunk.position[1] *
        this.terrain_params.chunk_size *
        this.terrain_params.tile_height;
      chunk_map.set(chunk_key, chunk_map_entry);
      renderer.layer_map[layer_kind].addChild(chunk_map_entry.container);
    }

    const chunk_map_entry = chunk_map.get(chunk_key)!;
    chunk_map_entry.data = chunk;
    chunk_map_entry.container.removeChildren();
    /*const graphics = new Graphics();
                        
                            graphics.lineStyle(1, 0xff0000);
                            graphics.drawRect(
                              chunk_map_entry.x,
                              chunk_map_entry.y,
                              chunk.width * chunk.width,
                              chunk.height * chunk.height
                            );
                        
                            renderer.debugContainer.addChild(graphics);*/

    for (const [i, gid] of chunk.data.entries()) {
      if (gid === 0) {
        continue;
      }

      const graphics = resource_manager.get_graphics_data_by_gid(gid);
      const sprite = resource_manager.get_sprite_from_graphics(graphics);

      sprite.x =
        (i % this.terrain_params.chunk_size) * this.terrain_params.tile_width;
      sprite.y =
        Math.floor(i / this.terrain_params.chunk_size) *
          this.terrain_params.tile_height +
        this.terrain_params.tile_height;
      sprite.rotation = 0;

      chunk_map_entry.container.addChild(sprite);
    }
  }

  private _toNatural(num: number): number {
    if (num < 0) {
      return -2 * num - 1;
    } else {
      return 2 * num;
    }
  }

  private _cantorPair(x: number, y: number): number {
    const xx = this._toNatural(x);
    const yy = this._toNatural(y);
    return ((xx + yy) * (xx + yy + 1)) / 2 + yy;
  }
}
