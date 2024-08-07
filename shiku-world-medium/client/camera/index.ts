import { Isometry } from "../entities";
import {
  get_simulation_scale,
  get_stage_height,
  get_stage_width,
} from "../config/config";
import { ParallaxContainer } from "../renderer/create_game_renderer";
import { CameraSettings } from "@/editor/blueprints/CameraSettings";
import { RenderGraphData } from "@/editor/stores/game-instances";
export function create_camera(): Camera {
  return new Camera();
}

export class Camera {
  private readonly _camera_isometry: Isometry;
  private _entity_id_ref: number | null = null;
  private _camera_settings: CameraSettings = {
    bounds: null,
    zoom: 1,
  };

  constructor() {
    this._camera_isometry = { x: 0, y: 0, rotation: 0 };
  }

  get zoom(): number {
    return this._camera_settings.zoom || 1;
  }

  get camera_isometry(): Isometry {
    return this._camera_isometry;
  }

  set_camera_ref(entity_id: number | null) {
    this._entity_id_ref = entity_id;
  }

  set_camera_settings(camera_settings: CameraSettings) {
    this._camera_settings = camera_settings;
    if (this._camera_settings.zoom) {
      this.set_camera_zoom(this._camera_settings.zoom);
    }
  }

  zoom_in() {
    this._camera_settings.zoom = 1 / (1 / this._camera_settings.zoom - 0.1);
  }

  zoom_out() {
    this._camera_settings.zoom = 1 / (1 / this._camera_settings.zoom + 0.1);
  }

  set_camera_zoom(zoom: number) {
    this._camera_settings.zoom = 1 / zoom;
  }

  update_camera_position(entity: { x: number; y: number; rotation: number }) {
    const iso = this._get_camera_iso(entity);
    if (this._camera_isometry.x != iso.x || this._camera_isometry.y != iso.y) {
      this._camera_isometry.x = iso.x;
      this._camera_isometry.y = iso.y;
    }
  }

  update_camera_position_from_ref(render_graph: RenderGraphData) {
    if (!this._entity_id_ref) {
      return;
    }
    const entity =
      render_graph.entity_node_to_render_node_map[this._entity_id_ref]
        ?.container;
    if (!entity) {
      return;
    }

    this.update_camera_position(entity);
  }

  private _get_camera_iso(
    position_object: { x: number; y: number; rotation: number } | undefined,
  ): Isometry {
    const iso = position_object || {
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
  zoom: number,
  container: ParallaxContainer,
) {
  const new_iso = camera_iso_to_scaled_viewport(
    camera_isometry,
    zoom,
    container,
  );
  container.x = new_iso.x;
  container.y = new_iso.y;
  container.rotation = new_iso.rotation;
}

export function camera_iso_to_scaled_viewport(
  camera_isometry: Isometry,
  zoom: number,
  { x_pscaling, y_pscaling }: { x_pscaling: number; y_pscaling: number },
): Isometry {
  return {
    x: -Math.round(
      camera_isometry.x * x_pscaling - (get_stage_width() * zoom) / 2,
    ),
    y: -Math.round(
      camera_isometry.y * y_pscaling - (get_stage_height() * zoom) / 2,
    ),
    rotation: camera_isometry.rotation,
  };
}
