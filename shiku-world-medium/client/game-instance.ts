import { InstanceRendering } from "@/client/renderer";
import { create_entity_manager, EntityManager } from "@/client/entities";
import { create_terrain_manager, TerrainManager } from "@/client/terrain";
import { create_instance_rendering } from "@/client/renderer/create_game_renderer";
import { GameSystemToGuestEvent } from "@/client/communication/api/bindings/GameSystemToGuestEvent";
import { match, P } from "ts-pattern";
import { MediumDataStorage } from "@/client/communication/api/bindings/MediumDataStorage";
import { get_plugin } from "@/client/plugins";
import { MousePluginType } from "../plugins/mouse-input";
import { new_shaker } from "@/client/renderer/shaker-factory";
import { MenuSystem } from "@/client/menu";
import { ResourceManager } from "@/client/resources";

export type GameInstanceMap = {
  [instance_id: string]: { [world_id: string]: GameInstance };
};

export class GameInstance {
  renderer: InstanceRendering;
  entity_manager: EntityManager;
  terrain_manager: TerrainManager;

  constructor(
    public id: string,
    public module_name: string,
    public world_id: string,
  ) {
    this.renderer = create_instance_rendering();
    this.entity_manager = create_entity_manager();
    this.terrain_manager = create_terrain_manager();
  }

  update() {
    this.renderer.camera.update_camera_position(this.entity_manager);

    /*for (const key in this.renderer.layerContainer) {
      const layerName = key as LayerName;
      if (layerName === "Menu") {
        continue;
      }
      const parallax_container = this.renderer.layerContainer[layerName];

      set_container_to_viewport_coordinate(
        this.camera.camera_isometry,
        parallax_container,
      );
    }*/
  }

  handle_position_update(
    position_update: Array<[string, number, number, number]>,
  ) {
    for (const [entity_id, x, y, r] of position_update) {
      this.entity_manager.update_entity_position(entity_id, x, y, r);
    }
  }

  handle_game_system_event(
    game_system_event: GameSystemToGuestEvent,
    menu_system: MenuSystem,
    resource_manager: ResourceManager,
  ) {
    match(game_system_event)
      .with({ SetCamera: P.select() }, ([entity_id, camera_settings]) => {
        this.renderer.camera.set_camera_ref(entity_id, this.module_name);
        this.renderer.camera.set_camera_settings(camera_settings);
      })
      .with({ OpenMenu: P.select() }, (menuName) => {
        menu_system.activate(menuName);
      })
      .with({ CloseMenu: P.select() }, (menuName) => {
        menu_system.deactivate(menuName);
      })
      .with({ ShowEntities: P.select() }, (show_entities) => {
        for (const show_entity of show_entities.filter(
          (s) => !s.parent_entity,
        )) {
          this.entity_manager.add_entity(
            show_entity,
            this.renderer,
            resource_manager,
          );
        }

        for (const show_entity of show_entities.filter(
          (s) => s.parent_entity,
        )) {
          this.entity_manager.add_entity(
            show_entity,
            this.renderer,
            resource_manager,
          );
        }
      })
      .with("RemoveAllEntities", () => {
        this.entity_manager.remove_all_entities_from_module();
        this.terrain_manager.remove_all_chunks_for_module(this.renderer);
      })
      .with({ RemoveEntities: P.select() }, (remove_entities) => {
        for (const remove_entity of remove_entities) {
          this.entity_manager.remove_entity(remove_entity);
        }
      })
      .with({ ChangeEntity: P.select() }, ([update_entities, _moduleName]) => {
        console.log(update_entities);
      })
      .with({ ShowTerrainChunks: P.select() }, ([tile_size, chunks]) => {
        for (const chunk of chunks) {
          this.terrain_manager.add_chunk(
            resource_manager,
            this.renderer,
            tile_size,
            chunk,
          );
        }
      })
      .with({ UpdateDataStore: P.select() }, (store_update) => {
        try {
          const update = JSON.parse(store_update) as MediumDataStorage;
          window.medium_gui.current_module.set_data(update);
        } catch (e) {
          console.error("Could not parse store update!");
        }
      })
      .with({ SetMouseInputSchema: P.select() }, (mouse_mode) => {
        const mouse_plugin = get_plugin("MOUSE") as MousePluginType | undefined;
        if (mouse_plugin) {
          mouse_plugin.plugin_options.mouse_mode = mouse_mode;
        }
      })
      .with({ ShowEffects: P.select() }, (show_effects) => {
        for (const show_effect of show_effects) {
          match(show_effect)
            .with({ SimpleImageEffect: P.select() }, (simple_image_effect) => {
              this.entity_manager.add_simple_image_effect(
                simple_image_effect,
                this.renderer,
                resource_manager,
              );
            })
            .with({ ShakeScreenEffect: P.select() }, (shake_screen_effect) => {
              new_shaker({
                target: this.renderer.mainContainer,
                isBidirectional: shake_screen_effect.is_bidirectional,
                shakeCountMax: shake_screen_effect.shake_count_max,
                shakeDelay: shake_screen_effect.shake_delay,
                shakeAmount: shake_screen_effect.shake_amount,
              }).shake();
            })
            .exhaustive();
        }
      })
      .with({ SetParallax: P.select() }, (_layer_parallax) => {
        /*for (const key in this.renderer.mainContainer) {
          const parallax_container =
            this.renderer.layerContainer[key as LayerName];
          parallax_container.y_pscaling = 1.0;
          parallax_container.x_pscaling = 1.0;
        }
        for (const [layer_name, parallax] of layer_parallax) {
          if (layer_name === "Menu") {
            continue;
          }
          this.renderer.layerContainer[layer_name].x_pscaling = parallax[0];
          this.renderer.layerContainer[layer_name].y_pscaling = parallax[1];
        }*/
      })
      .with({ UpdateEntities: P.select() }, (updated_entities) => {
        for (const update_entity of updated_entities) {
          this.entity_manager.update_entity(
            update_entity,
            resource_manager,
            this.renderer,
          );
        }
      })
      .exhaustive();
  }

  destroy() {}
}

export function create_new_game_instance(
  id: string,
  module_name: string,
  world_id: string,
) {
  return new GameInstance(id, module_name, world_id);
}
