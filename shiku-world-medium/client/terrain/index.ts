import { Container } from "pixi.js";
import { ResourceManager } from "../resources";
import { InstanceRendering } from "../renderer";
import { TerrainParams } from "@/editor/blueprints/TerrainParams";
import { LayerKind } from "@/editor/blueprints/LayerKind";
import { Chunk } from "@/editor/blueprints/Chunk";
import { EffectsManager } from "@/client/effects-manager";

export function to_natural(num: number): number {
  if (num < 0) {
    return -2 * num - 1;
  } else {
    return 2 * num;
  }
}

export function cantor_pair(x: number, y: number): number {
  const xx = to_natural(x);
  const yy = to_natural(y);
  return ((xx + yy) * (xx + yy + 1)) / 2 + yy;
}

export function create_terrain_manager(
  terrain_params: TerrainParams,
): TerrainManager {
  return new TerrainManager(terrain_params);
}

export class TerrainManager {
  private _chunk_map: Map<
    LayerKind,
    Map<number, { container: Container; data: Chunk }>
  > = new Map();
  sprite_effects_manager = new EffectsManager();

  constructor(public terrain_params: TerrainParams) {}

  destroy() {}

  remove_all_chunks_for_module(renderer: InstanceRendering) {
    for (const [layer, layer_chunks] of this._chunk_map.entries()) {
      for (const chunk of Object.values(layer_chunks)) {
        renderer.layer_map[layer].removeChild(chunk.container);
      }
    }
  }

  update_animations_for_animated_sprites(
    resource_manager: ResourceManager,
    gid: number,
  ) {
    this.sprite_effects_manager.update_animations_for_animated_sprites(
      resource_manager,
      gid,
    );
  }

  sync_sprite_animations() {
    this.sprite_effects_manager.sync_sprite_animations();
  }

  update() {
    this.sprite_effects_manager.sync_sprite_animations();
    this.sprite_effects_manager.update_effects();
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
    const chunk_key = cantor_pair(chunk.position[0], chunk.position[1]);
    if (!chunk_map.has(chunk_key)) {
      const chunk_map_entry = {
        container: new Container(),
        data: chunk,
        animations: {},
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
      const x =
        (i % this.terrain_params.chunk_size) * this.terrain_params.tile_width;
      const y =
        Math.floor(i / this.terrain_params.chunk_size) *
          this.terrain_params.tile_height +
        this.terrain_params.tile_height;
      const tile_key = get_tile_key(
        layer_kind,
        chunk.position[0],
        chunk.position[1],
        x,
        y,
      );
      const sprite_effect =
        this.sprite_effects_manager.get_sprite_effect_by_unique_key(tile_key);
      if (!sprite_effect) {
        if (gid === 0) {
          continue;
        }
        this._create_new_tile(
          resource_manager,
          gid,
          x + this.terrain_params.tile_width / 2,
          y - this.terrain_params.tile_height / 2,
          tile_key,
          chunk_map_entry,
        );
      } else {
        if (gid === sprite_effect.gid) {
          continue;
        }
        this.sprite_effects_manager.remove_sprite_effect(tile_key);
        sprite_effect.fade_out.tween.start(window.performance.now());
        sprite_effect.fade_out.all_tweens[
          sprite_effect.fade_out.all_tweens.length - 1
        ].onComplete(() => {
          chunk_map_entry.container.removeChild(sprite_effect.sprite);
        });
        if (gid !== 0) {
          this._create_new_tile(
            resource_manager,
            gid,
            x + this.terrain_params.tile_width / 2,
            y - this.terrain_params.tile_height / 2,
            tile_key,
            chunk_map_entry,
          );
        }
      }
    }
  }

  private _create_new_tile(
    resource_manager: ResourceManager,
    gid: number,
    x: number,
    y: number,
    tile_key: string,
    chunk_map_entry: {
      container: Container;
      data: Chunk;
    },
  ) {
    const graphics = resource_manager.get_graphics_data_by_gid(gid);
    const sprite = resource_manager.get_animated_sprite_from_graphics(graphics);
    sprite.x = x;
    sprite.y = y;
    sprite.rotation = 0;
    chunk_map_entry.container.addChild(sprite);

    this.sprite_effects_manager.add_sprite_with_effects(sprite, tile_key, gid);
  }
}

function get_tile_key(
  layer_kind: LayerKind,
  chunk_x: number,
  chunk_y: number,
  tile_x: number,
  tile_y: number,
): string {
  return `${layer_kind}_${chunk_x}_${chunk_y}_${tile_x}_${tile_y}`;
}
