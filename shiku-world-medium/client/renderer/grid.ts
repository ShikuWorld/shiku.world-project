import {
  TextureSource,
  Container,
  Graphics,
  RenderTexture,
  TilingSprite,
} from "pixi.js";
import { InstanceRendering, RenderSystem } from "@/client/renderer/index";
import { Isometry } from "@/client/entities";
import { camera_iso_to_scaled_viewport } from "@/client/camera";
import { match } from "ts-pattern";
import { SimpleEventDispatcher } from "strongly-typed-events";

export function adjust_selected_tile_size(
  renderer: InstanceRendering,
  brush: number[][],
) {
  if (renderer.grid && brush.length > 0 && brush[0].length > 0) {
    const height = brush.length;
    const width = brush[0].length;
    renderer.grid.grid_container.removeChild(renderer.grid.selected_tile);
    renderer.grid.selected_tile = new Graphics()
      .rect(
        0,
        0,
        renderer.terrain_params.tile_width * width,
        renderer.terrain_params.tile_height * height,
      )
      .fill({
        color: "#9999ff",
        alpha: 0.5,
      });
    renderer.grid.grid_container.addChild(renderer.grid.selected_tile);
  }
}

export function init_grid(
  renderer_system: RenderSystem,
  renderer: InstanceRendering,
) {
  if (!renderer.grid) {
    const textureSource = new TextureSource({
      width: renderer.terrain_params.tile_width,
      height: renderer.terrain_params.tile_height,
    });
    const renderTexture = new RenderTexture({ source: textureSource });
    const graphics = new Graphics()
      .rect(0, 0, 1, renderer.terrain_params.tile_height)
      .rect(0, 0, renderer.terrain_params.tile_width, 1)
      .fill({
        color: "#ffffff",
        alpha: 0.5,
      });
    renderer_system.renderer.render({
      target: renderTexture,
      container: graphics,
    });
    const selected_tile = new Graphics()
      .rect(
        0,
        0,
        renderer.terrain_params.tile_width,
        renderer.terrain_params.tile_height,
      )
      .fill({
        color: "#9999ff",
        alpha: 0.5,
      });
    const grid: InstanceRendering["grid"] = {
      sprite: new TilingSprite({
        texture: renderTexture,
        height: 1440,
        width: 2560,
      }),
      mouse_wheel_event: new SimpleEventDispatcher<number>(),
      grid_container: new Container(),
      p_scaling: { x: 1, y: 1 },
      last_mouse_move_position: { x: 0, y: 0 },
      selected_tile,
    };
    let middle_click_start: {
      click_start: { x: number; y: number };
      camera_iso_start: Isometry;
    } | null = null;
    let left_click_down = false;
    grid.grid_container.addChild(grid.sprite);
    grid.grid_container.interactive = true;
    grid.grid_container.on("pointerdown", (mouse_event) => {
      match(mouse_event.button)
        .with(0, () => {
          const [x, y] = [
            -(grid.sprite.tilePosition.x - grid.selected_tile.x) /
              renderer.terrain_params.tile_width,
            -(grid.sprite.tilePosition.y - grid.selected_tile.y) /
              renderer.terrain_params.tile_height,
          ];
          left_click_down = true;
          window.medium_gui.editor.select_tile_position({
            x: x,
            y: y - 1,
          });
        })
        .with(1, () => {
          middle_click_start = {
            click_start: { x: mouse_event.x, y: mouse_event.y },
            camera_iso_start: { ...renderer.camera.camera_isometry },
          };
        })
        .otherwise(() => {});
    });

    grid.grid_container.on("wheel", (wheel) => {
      grid.mouse_wheel_event.dispatch(wheel.deltaY);
    });

    grid.grid_container.on("pointerup", (mouse_event) => {
      match(mouse_event.button)
        .with(0, () => {
          left_click_down = false;
        })
        .with(1, () => {
          middle_click_start = null;
        })
        .otherwise(() => {});
    });

    grid.grid_container.addChild(selected_tile);
    renderer_system.global_mouse_position.sub((mouse_event) => {
      if (middle_click_start) {
        const diff = {
          x: mouse_event.x - middle_click_start.click_start.x,
          y: mouse_event.y - middle_click_start.click_start.y,
        };
        renderer.camera.update_camera_position({
          x: middle_click_start.camera_iso_start.x - diff.x,
          y: middle_click_start.camera_iso_start.y - diff.y,
          rotation: middle_click_start.camera_iso_start.rotation,
        });
      }
      grid.last_mouse_move_position = mouse_event;
      const new_selected_tile_position =
        get_current_selected_tile_position(renderer);

      if (new_selected_tile_position !== null) {
        const [new_selected_tile_x, new_selected_tile_y] =
          new_selected_tile_position;
        if (
          new_selected_tile_x != grid.selected_tile.x ||
          new_selected_tile_y != grid.selected_tile.y
        ) {
          if (left_click_down) {
            const [x, y] = [
              -(grid.sprite.tilePosition.x - new_selected_tile_x) /
                renderer.terrain_params.tile_width,
              -(grid.sprite.tilePosition.y - new_selected_tile_y) /
                renderer.terrain_params.tile_height,
            ];
            window.medium_gui.editor.select_tile_position({
              x,
              y: y - 1,
            });
          }
          grid.selected_tile.x = new_selected_tile_x;
          grid.selected_tile.y = new_selected_tile_y;
        }
      }
    });
    renderer.grid = grid;
    renderer.main_container.addChild(renderer.grid.grid_container);
  }
}

export function toggle_grid(renderer: InstanceRendering) {
  if (renderer.grid) {
    renderer.grid.sprite.alpha = renderer.grid.sprite.alpha === 0 ? 1 : 0;
  }
}

export function get_current_selected_tile_position(
  renderer: InstanceRendering,
): [number, number] | null {
  const grid = renderer.grid;
  if (!grid) {
    return null;
  }
  const localPosition = grid.grid_container.toLocal(grid.sprite.tilePosition);

  const [local_x, local_y] = [
    Math.round(localPosition.x * (1 / renderer.camera.zoom)),
    Math.round(localPosition.y * (1 / renderer.camera.zoom)),
  ];
  const diff = {
    x: local_x - grid.last_mouse_move_position.x * renderer.camera.zoom,
    y: local_y - grid.last_mouse_move_position.y * renderer.camera.zoom,
  };
  const selected_new_x =
    local_x -
    Math.floor(
      (diff.x + renderer.terrain_params.tile_width) /
        renderer.terrain_params.tile_width,
    ) *
      renderer.terrain_params.tile_width;
  const selected_new_y =
    local_y -
    Math.floor(
      (diff.y + renderer.terrain_params.tile_height) /
        renderer.terrain_params.tile_height,
    ) *
      renderer.terrain_params.tile_height;
  return [selected_new_x, selected_new_y];
}

export function update_grid(
  camera_isometry: Isometry,
  renderer: InstanceRendering,
) {
  if (renderer.grid) {
    const new_iso = camera_iso_to_scaled_viewport(camera_isometry, {
      y_pscaling: renderer.grid.p_scaling.y,
      x_pscaling: renderer.grid.p_scaling.x,
    });

    if (
      renderer.grid.sprite.tilePosition.x !== new_iso.x ||
      renderer.grid.sprite.tilePosition.y !== new_iso.y
    ) {
      renderer.grid.sprite.tilePosition.x = new_iso.x;
      renderer.grid.sprite.tilePosition.y = new_iso.y;
    }
  }
}
