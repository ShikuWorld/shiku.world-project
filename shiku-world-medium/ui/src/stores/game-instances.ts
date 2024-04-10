import { defineStore } from "pinia";
import { Entity } from "@/editor/blueprints/Entity";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { Scene } from "@/editor/blueprints/Scene";
import { Container } from "pixi.js";
import { create_display_object, ResourceManager } from "@/client/resources";
import { get_generic_game_node } from "@/editor/stores/resources";
import { EntityUpdate } from "@/editor/blueprints/EntityUpdate";
import { match, P } from "ts-pattern";
import { Node2D } from "@/editor/blueprints/Node2D";
import { GameNode } from "@/editor/blueprints/GameNode";
import { toRaw } from "vue";

export interface Node {
  node_id: string | Entity;
  parent: Node | null;
  children: Node[];
  container: Container;
}

export interface RenderGraphData {
  render_root: Node;
  entity_node_to_render_node_map: { [key: string | Entity]: Node };
  entity_node_map: { [key: string | Entity]: GameNodeKind };
}
export interface GameInstancesStore {
  game_instance_data_map: {
    [instance_id: string]: {
      [world_id: string]: {
        render_graph_data: RenderGraphData;
        instance_scene: Scene | null;
      };
    };
  };
  blueprint_render: {
    render_graph_data?: RenderGraphData;
    scene_resource_path: string;
    is_pinned: boolean;
  } | null;
}

export const use_game_instances_store = defineStore("game-instances", {
  state: (): GameInstancesStore => ({
    game_instance_data_map: {},
    blueprint_render: null,
  }),
  actions: {
    set_and_render_blueprint_render(
      module_id: string,
      scene_resource_path: string,
      scene: Scene,
      is_pinned: boolean,
    ) {
      const resource_module = window.medium.get_resource_manager(module_id);
      if (!resource_module) {
        return;
      }

      const render_graph_data =
        this.blueprint_render?.scene_resource_path !== scene_resource_path
          ? this.render_graph_from_scene(
              scene,
              resource_module,
              window.medium.create_display_object,
            )
          : this.blueprint_render.render_graph_data;

      this.blueprint_render = {
        scene_resource_path,
        render_graph_data,
        is_pinned,
      };

      if (!this.blueprint_render.render_graph_data?.render_root.container) {
        return;
      }

      window.medium.set_blueprint_renderer(
        this.blueprint_render as GameInstancesStore["blueprint_render"],
      );
    },
    add_game_instance_data(
      instance_id: string,
      world_id: string,
      create_container: () => Container,
    ) {
      if (!this.game_instance_data_map[instance_id]) {
        this.game_instance_data_map[instance_id] = {};
      }
      this.game_instance_data_map[instance_id][world_id] = {
        render_graph_data: {
          render_root: {
            node_id: 0,
            children: [],
            container: create_container(),
            parent: null,
          },
          entity_node_map: {},
          entity_node_to_render_node_map: {},
        },
        instance_scene: null,
      };
    },
    get_raw_root_container(instance_id: string, world_id: string) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      if (!game_instance_data?.render_graph_data) {
        return null;
      }
      return toRaw(game_instance_data.render_graph_data.render_root.container);
    },
    get_game_instance_data(instance_id: string, world_id: string) {
      if (
        !this.game_instance_data_map[instance_id] ||
        !this.game_instance_data_map[instance_id][world_id]
      ) {
        console.error(
          `Could not get render graph of ${instance_id} ${world_id}`,
        );
        return null;
      }
      return this.game_instance_data_map[instance_id][world_id];
    },
    game_instance_exists(instance_id: string, world_id: string): boolean {
      return (
        !!this.game_instance_data_map[instance_id] &&
        !!this.game_instance_data_map[instance_id][world_id]
      );
    },
    render_graph_from_scene_for_instance(
      instance_id: string,
      world_id: string,
      instance_scene: Scene,
      resource_manager: ResourceManager,
      create_display_object_cb: typeof create_display_object,
    ) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      if (!game_instance_data?.render_graph_data) {
        return;
      }
      game_instance_data.instance_scene = instance_scene;
      game_instance_data.render_graph_data = this.render_graph_from_scene(
        instance_scene,
        resource_manager,
        create_display_object_cb,
      );
    },
    render_graph_from_scene(
      scene: Scene,
      resource_manager: ResourceManager,
      create_display_object_cb: typeof create_display_object,
    ): RenderGraphData {
      const game_node_root = get_generic_game_node(scene.root_node);
      const render_graph_data: RenderGraphData = {
        render_root: {
          node_id: game_node_root.id,
          children: [],
          container: create_display_object_cb(
            scene.root_node,
            resource_manager,
          ),
          parent: null,
        },
        entity_node_map: {},
        entity_node_to_render_node_map: {},
      };
      render_graph_data.entity_node_to_render_node_map[
        render_key(game_node_root)
      ] = render_graph_data.render_root;
      render_graph_data.entity_node_map[render_key(game_node_root)] =
        scene.root_node;

      this.generate_render_graph(
        render_graph_data.entity_node_to_render_node_map,
        render_graph_data.entity_node_map,
        render_graph_data.render_root,
        scene.root_node,
        resource_manager,
        create_display_object_cb,
      );
      return render_graph_data;
    },
    apply_entity_update(
      instance_id: string,
      world_id: string,
      update: EntityUpdate,
      resource_manager: ResourceManager,
    ) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      if (!game_instance_data) {
        return;
      }
      const render_graph_data = game_instance_data.render_graph_data;
      const node = render_graph_data.entity_node_map[update.id];
      const render_node =
        render_graph_data.entity_node_to_render_node_map[update.id];
      if (!node || !render_node) {
        console.error("Could not update game node!");
        return;
      }
      console.log(update);
      const game_node = Object.values(node)[0];
      match(update.kind)
        .with({ UpdateTransform: P.select() }, (transform) => {
          if (!game_node.data.transform) {
            console.error("Tried to update Node without transform, wtf?");
            return;
          }
          (game_node.data as Node2D).transform = transform;
          render_node.container.position.x = transform.position[0];
          render_node.container.position.y = transform.position[1];
          render_node.container.rotation = transform.rotation;
        })
        .with({ UpdateGid: P.select() }, (gid) => {
          if (get_gid(game_node) === gid) {
            return;
          }
          const graphics = resource_manager.get_graphics_data_by_gid(gid);
          render_node.container.removeChildAt(0);
          render_node.container.addChildAt(
            resource_manager.get_sprite_from_graphics(graphics),
            0,
          );
        })
        .exhaustive();
    },
    generate_render_graph(
      entity_node_to_render_node_map: RenderGraphData["entity_node_to_render_node_map"],
      entity_node_map: RenderGraphData["entity_node_map"],
      parent: Node,
      game_node: GameNodeKind,
      resource_manager: ResourceManager,
      create_display_object: (
        node: GameNodeKind,
        resource_manager: ResourceManager,
      ) => Container,
    ) {
      const generic_game_node = get_generic_game_node(game_node);
      for (const game_node_child of generic_game_node.children) {
        const generic_game_node_child = get_generic_game_node(game_node_child);
        const new_node = {
          children: [],
          parent,
          container: create_display_object(game_node_child, resource_manager),
          scene_node: game_node,
          node_id:
            generic_game_node_child.entity_id || generic_game_node_child.id,
        };
        parent.children.push(new_node);
        parent.container.addChild(new_node.container);
        entity_node_to_render_node_map[new_node.node_id] = new_node;
        entity_node_map[new_node.node_id] = game_node_child;
        this.generate_render_graph(
          entity_node_to_render_node_map,
          entity_node_map,
          new_node,
          game_node_child,
          resource_manager,
          create_display_object,
        );
      }
    },
  },
});

function get_gid(game_node: GameNodeKind): number | undefined {
  if ("Node2D" in game_node) {
    const node_2d = game_node.Node2D;
    if ("Render" in node_2d.data.kind) {
      return match(node_2d.data.kind.Render.kind)
        .with({ Sprite: P.select() }, (gid) => gid)
        .with({ AnimatedSprite: P.select() }, (gid) => gid)
        .exhaustive();
    }
  }

  return undefined;
}

function render_key(game_node: GameNode<unknown>) {
  return typeof game_node.entity_id === "number"
    ? game_node.entity_id
    : game_node.id;
}
