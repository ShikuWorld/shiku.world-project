import { match, P } from "ts-pattern";
import { EditorEvent } from "@/client/communication/api/bindings/EditorEvent";
import { GameMap } from "@/editor/blueprints/GameMap";

export function handle_editor_event(event: EditorEvent) {
  match(event)
    .with({ MainDoorStatus: P.select() }, (status) => {
      window.medium_gui.editor.set_main_door_status(status);
    })
    .with({ Modules: P.select() }, (modules) => {
      window.medium_gui.resources.set_modules(modules);
    })
    .with({ CreatedModule: P.select() }, (d) => {
      window.medium_gui.resources.create_module(d[1]);
    })
    .with({ UpdatedModule: P.select() }, (d) => {
      window.medium_gui.resources.update_module(d[1]);
    })
    .with({ DeletedModule: P.select() }, (d) => {
      window.medium_gui.resources.delete_module(d);
    })
    .with({ CreatedTileset: P.select() }, (d) => {
      window.medium_gui.resources.set_tileset(d);
    })
    .with({ DirectoryInfo: P.select() }, (d) => {
      window.medium_gui.resources.set_current_file_browser_result(d);
    })
    .with({ SetTileset: P.select() }, (d) => {
      window.medium_gui.resources.set_tileset(d);
    })
    .with({ DeletedTileset: P.select() }, (d) => {
      window.medium_gui.resources.delete_tileset(d);
    })
    .with({ CreatedScene: P.select() }, (d) => {
      window.medium_gui.resources.set_scene(d);
    })
    .with({ SetScene: P.select() }, (d) => {
      window.medium_gui.resources.set_scene(d);
    })
    .with({ DeletedScene: P.select() }, (d) => {
      window.medium_gui.resources.delete_scene(d);
    })
    .with({ CreatedMap: P.select() }, (d) => {
      window.medium_gui.resources.set_map(d);
    })
    .with({ SetMap: P.select() }, (d) => {
      window.medium_gui.resources.set_map(d);
    })
    .with({ UpdatedMap: P.select() }, (d) => {
      window.medium_gui.resources.update_map(
        d as Partial<GameMap> & { resource_path: string; name: string },
      );
    })
    .with({ DeletedMap: P.select() }, (d) => {
      window.medium_gui.resources.delete_map(d);
    })
    .with({ UpdatedConductor: P.select() }, (d) => {
      window.medium_gui.editor.set_conductor(d);
    })
    .with({ ModuleInstances: P.select() }, (d) => {
      window.medium_gui.editor.set_game_instance_map(d);
    })
    .with(
      { ModuleInstanceOpened: P.select() },
      ([module_id, game_instance_id]) => {
        window.medium_gui.editor.add_module_instance(
          module_id,
          game_instance_id,
        );
      },
    )
    .with(
      { ModuleInstanceClosed: P.select() },
      ([module_id, game_instance_id]) => {
        window.medium_gui.editor.remove_module_instance(
          module_id,
          game_instance_id,
        );
      },
    )
    .exhaustive();
}
