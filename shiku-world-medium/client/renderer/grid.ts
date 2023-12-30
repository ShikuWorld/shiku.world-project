import {
  BaseRenderTexture,
  Container,
  Graphics,
  RenderTexture,
  TilingSprite,
} from "pixi.js-legacy";
import { InstanceRendering, RenderSystem } from "@/client/renderer/index";
import { Isometry } from "@/client/entities";
import { camera_iso_to_scaled_viewport } from "@/client/camera";

export function show_grid(
  renderer_system: RenderSystem,
  renderer: InstanceRendering,
) {
  if (!renderer.grid) {
    const baseRenderTexture = new BaseRenderTexture({
      width: renderer.terrain_params.tile_width,
      height: renderer.terrain_params.tile_height,
    });
    const renderTexture = new RenderTexture(baseRenderTexture);
    const graphics = new Graphics();
    graphics.beginFill([1.0, 1.0, 1.0, 0.5]);
    graphics.drawRect(0, 0, 1, renderer.terrain_params.tile_height);
    graphics.drawRect(0, 0, renderer.terrain_params.tile_width, 1);
    graphics.endFill();
    renderer_system.renderer.render(graphics, { renderTexture });
    const selected_tile = new Graphics();
    selected_tile.beginFill([0.7, 0.7, 1.0, 0.5]);
    selected_tile.drawRect(
      0,
      0,
      renderer.terrain_params.tile_width,
      renderer.terrain_params.tile_height,
    );
    selected_tile.endFill();
    const grid = {
      sprite: new TilingSprite(
        renderTexture,
        renderer.terrain_params.tile_height * 100,
        renderer.terrain_params.tile_width * 100,
      ),
      grid_container: new Container(),
      scaling: { x: 1, y: 1 },
      last_mouse_move_position: { x: 0, y: 0 },
      selected_tile,
    };

    grid.grid_container.addChild(grid.sprite);

    grid.grid_container.interactive = true;
    grid.grid_container.on("pointerdown", () => {
      const [x, y] = [
        -(grid.sprite.tilePosition.x - grid.selected_tile.x) /
          renderer.terrain_params.tile_width,
        -(grid.sprite.tilePosition.y - grid.selected_tile.y) /
          renderer.terrain_params.tile_height,
      ];
      window.medium_gui.editor.selected_tile_position = { x, y };
    });

    grid.grid_container.addChild(selected_tile);
    renderer_system.global_mouse_position.sub((mouse_event) => {
      grid.last_mouse_move_position = mouse_event;
      update_tile_position(renderer);
    });
    renderer.grid = grid;
  }
  renderer.main_container.addChild(renderer.grid.grid_container);
}

export function update_tile_position(renderer: InstanceRendering) {
  const grid = renderer.grid;
  if (!grid) {
    return;
  }
  const localPosition = grid.grid_container.toLocal(grid.sprite.tilePosition);
  const diff = {
    x: localPosition.x - grid.last_mouse_move_position.x,
    y: localPosition.y - grid.last_mouse_move_position.y,
  };
  grid.selected_tile.x =
    localPosition.x -
    Math.floor(
      (diff.x + renderer.terrain_params.tile_width) /
        renderer.terrain_params.tile_width,
    ) *
      renderer.terrain_params.tile_width;
  grid.selected_tile.y =
    localPosition.y -
    Math.floor(
      (diff.y + renderer.terrain_params.tile_height) /
        renderer.terrain_params.tile_height,
    ) *
      renderer.terrain_params.tile_height;
}

export function update_grid(
  camera_isometry: Isometry,
  renderer: InstanceRendering,
) {
  if (renderer.grid) {
    const new_iso = camera_iso_to_scaled_viewport(camera_isometry, {
      y_pscaling: renderer.grid.scaling.y,
      x_pscaling: renderer.grid.scaling.x,
    });
    renderer.grid.sprite.tilePosition.x = new_iso.x;
    renderer.grid.sprite.tilePosition.y = new_iso.y;

    update_tile_position(renderer);
  }
}
export function hide_grid(renderer: InstanceRendering) {
  if (renderer.grid) {
    renderer.main_container.removeChild(renderer.grid.sprite);
  }
}
