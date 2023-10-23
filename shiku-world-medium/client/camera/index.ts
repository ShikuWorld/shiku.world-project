import { Entity, EntityManager, Isometry } from "../entities";
import {
  get_simulation_scale,
  get_stage_height,
  get_stage_width,
  set_camera_zoom,
} from "../config/config";
import {
  ParallaxContainer,
  viewPortResize,
} from "../renderer/create_game_renderer";
import { RenderSystem } from "../renderer";
import { CameraSettings } from "../communication/api/bindings/CameraSettings";

export function create_camera(): Camera {
  return new Camera();
}

export class Camera {
  private _isometry_ref: Isometry;
  private readonly _camera_isometry: Isometry;
  private _entity_id_ref: { module_name: string; entity_id: string };
  private _camera_settings: CameraSettings = {
    bounds: [
      [0, 0],
      [0, 0],
    ],
    zoom: 1,
  };

  constructor() {
    this._camera_isometry = { x: 0, y: 0, rotation: 0 };
    this._isometry_ref = { x: 0, y: 0, rotation: 0 };
    this._entity_id_ref = { entity_id: "", module_name: "" };
  }

  get camera_isometry(): Isometry {
    return this._camera_isometry;
  }

  set_camera_ref(entity_id: string, module_name: string) {
    this._entity_id_ref = {
      entity_id,
      module_name,
    };
  }

  set_camera_settings(camera_settings: CameraSettings, renderer: RenderSystem) {
    this._camera_settings = camera_settings;
    if (this._camera_settings.zoom) {
      const canvas_wrapper = document.getElementById("canvas") as HTMLElement;
      const width = canvas_wrapper.offsetWidth;
      const height = canvas_wrapper.offsetHeight;
      set_camera_zoom(this._camera_settings.zoom);
      viewPortResize(width, height, renderer);
    }
  }

  update_camera_position(entities: EntityManager, renderer: RenderSystem) {
    if (!this._entity_id_ref.module_name) {
      return;
    }

    const iso = this._get_camera_iso(
      entities.get_entity(this._entity_id_ref.entity_id),
    );

    if (this._camera_isometry.x != iso.x || this._camera_isometry.y != iso.y) {
      this._camera_isometry.x = iso.x;
      this._camera_isometry.y = iso.y;
      renderer.isDirty = true;
    }
  }

  private _get_camera_iso(entity: Entity | undefined): Isometry {
    const iso = entity?.wrapper || {
      x: 0,
      y: 0,
      rotation: 0,
    };

    if (this._camera_settings && this._camera_settings.bounds !== null) {
      const p_min = this._camera_settings.bounds[0];
      const p_max = this._camera_settings.bounds[1];
      const simulation_scale = get_simulation_scale();

      const boundary_set = {
        min: {
          x: p_min[0] * simulation_scale + get_stage_width() / 2,
          y: p_min[1] * simulation_scale + get_stage_height() / 2,
        },
        max: {
          x: p_max[0] * simulation_scale - get_stage_width() / 2,
          y: p_max[1] * simulation_scale - get_stage_height() / 2,
        },
      };

      return {
        x: Math.max(boundary_set.min.x, Math.min(iso.x, boundary_set.max.x)),
        y: Math.max(boundary_set.min.y, Math.min(iso.y, boundary_set.max.y)),
        rotation: iso.rotation,
      };
    }

    return iso;
  }
}

export function set_container_to_viewport_coordinate(
  camera_isometry: Isometry,
  container: ParallaxContainer,
) {
  container.x = -Math.round(
    camera_isometry.x * container.x_pscaling - get_stage_width() / 2,
  );
  container.y = -Math.round(
    camera_isometry.y * container.y_pscaling - get_stage_height() / 2,
  );
  container.rotation = camera_isometry.rotation;
}
