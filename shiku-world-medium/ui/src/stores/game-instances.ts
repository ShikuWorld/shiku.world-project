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
import { markRaw } from "vue";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { RENDER_SCALE } from "@/shared/index";

export interface Node {
  node_id: ReturnType<typeof render_key>;
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
    module_id: string;
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
        module_id,
      };

      if (!this.blueprint_render.render_graph_data?.render_root.container) {
        return;
      }
      console.log(this.blueprint_render);
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
            container: markRaw(create_container()),
            parent: null,
          },
          entity_node_map: {},
          entity_node_to_render_node_map: {},
        },
        instance_scene: null,
      };
    },
    get_root_container(instance_id: string, world_id: string) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      if (!game_instance_data?.render_graph_data) {
        return null;
      }
      return game_instance_data.render_graph_data.render_root.container;
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
          node_id: render_key(game_node_root),
          children: [],
          container: markRaw(
            create_display_object_cb(scene.root_node, resource_manager),
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
    remove_entity_from_instance(
      instance_id: string,
      world_id: string,
      entity: Entity,
    ) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      if (!game_instance_data) {
        return;
      }
      const render_graph_data = game_instance_data.render_graph_data;
      this.remove_child_from_render_graph(render_graph_data, entity);
    },
    add_entity_to_instance(
      instance_id: string,
      world_id: string,
      parent_id: Entity,
      node_to_insert: GameNodeKind,
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
      this.add_child_to_render_graph(
        render_graph_data,
        parent_id,
        node_to_insert,
        resource_manager,
        window.medium.create_display_object,
      );
    },
    apply_entity_update_for_instance(
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
      this.apply_entity_update(render_graph_data, update, resource_manager);
    },
    apply_entity_update(
      render_graph_data: RenderGraphData,
      update: { id: string | number; kind: EntityUpdateKind },
      resource_manager: ResourceManager,
    ) {
      const node = render_graph_data.entity_node_map[update.id];
      const render_node =
        render_graph_data.entity_node_to_render_node_map[update.id];
      if (!node || !render_node) {
        console.error("Could not update game node!");
        return;
      }
      const game_node = Object.values(node)[0];
      match(update.kind)
        .with({ Transform: P.select() }, (transform) => {
          if (!game_node.data.transform) {
            console.error("Tried to update Node without transform, wtf?");
            return;
          }
          (game_node.data as Node2D).transform = transform;
          render_node.container.position.x =
            transform.position[0] * RENDER_SCALE;
          render_node.container.position.y =
            transform.position[1] * RENDER_SCALE;
          render_node.container.rotation = transform.rotation;
        })
        .with({ ScriptPath: P.select() }, (script) => {
          game_node.script = script;
        })
        .with({ Name: P.select() }, (name) => {
          game_node.name = name;
        })
        .with({ RigidBodyType: P.select() }, (rigid_body_type) => {
          if (!game_node.data.transform) {
            console.error(
              "Tried to update rigid body without a transform, wtf?",
            );
            return;
          }
          const node_2d = game_node.data as Node2D;
          const rigid_body =
            "RigidBody" in node_2d.kind ? node_2d.kind.RigidBody : null;
          if (!rigid_body) {
            console.error("Could not upate rigid body type");
            return;
          }
          rigid_body.body = rigid_body_type;
        })
        .with({ PositionRotation: P.select() }, ([x, y, r]) => {
          if (!game_node.data.transform) {
            console.error("Tried to update Node without transform, wtf?");
            return;
          }
          const node_2d = game_node.data as Node2D;
          node_2d.transform.position = [x, y];
          node_2d.transform.rotation = r;
          render_node.container.position.x = x * RENDER_SCALE;
          render_node.container.position.y = y * RENDER_SCALE;
          render_node.container.rotation = r;
        })
        .with({ Gid: P.select() }, (gid) => {
          if (get_gid(game_node) === gid) {
            return;
          }
          match((game_node.data as Node2D).kind)
            .with({ Render: { kind: P.select() } }, (render_kind) => {
              match(render_kind)
                .with({ Sprite: P.select() }, () => {
                  (render_kind as { Sprite: number }).Sprite = gid;
                })
                .with({ AnimatedSprite: P.select() }, () => {
                  (render_kind as { AnimatedSprite: number }).AnimatedSprite =
                    gid;
                })
                .exhaustive();
            })
            .run();
          const graphics = resource_manager.get_graphics_data_by_gid(gid);
          render_node.container.removeChildAt(0);
          render_node.container.addChildAt(
            resource_manager.get_sprite_from_graphics(graphics),
            0,
          );
        })
        .exhaustive();
    },
    add_child_to_render_graph(
      render_graph_data: RenderGraphData,
      parent_node_id: string | number,
      node_to_insert: GameNodeKind,
      resource_manager: ResourceManager,
      create_display_object_cb: typeof create_display_object,
    ) {
      const parent_node_render_node =
        render_graph_data.entity_node_to_render_node_map[parent_node_id];
      const parent_node_game_node =
        render_graph_data.entity_node_map[parent_node_id];
      if (!parent_node_render_node || !parent_node_game_node) {
        console.error("Could not add child to node!");
        return;
      }
      this.add_node_to_graph(
        render_graph_data.entity_node_to_render_node_map,
        render_graph_data.entity_node_map,
        parent_node_render_node,
        node_to_insert,
        resource_manager,
        create_display_object_cb,
      );
      get_generic_game_node(parent_node_game_node).children.push(
        node_to_insert,
      );
      this.generate_render_graph(
        render_graph_data.entity_node_to_render_node_map,
        render_graph_data.entity_node_map,
        parent_node_render_node,
        node_to_insert,
        resource_manager,
        create_display_object_cb,
      );
    },
    remove_child_from_render_graph(
      render_graph_data: RenderGraphData,
      node_to_remove_id: ReturnType<typeof render_key>,
    ) {
      const node_to_remove =
        render_graph_data.entity_node_to_render_node_map[node_to_remove_id];
      const parent_node_render_node = node_to_remove?.parent;
      if (!parent_node_render_node || !node_to_remove) {
        console.error("Could not remove child from node!");
        return;
      }
      const parent_node_game_node =
        render_graph_data.entity_node_map[parent_node_render_node.node_id];
      if (!parent_node_game_node) {
        console.error("Could not fine parent node to remove from!");
        return;
      }
      const generic_parent_game_node = get_generic_game_node(
        parent_node_game_node,
      );
      if (!node_to_remove) {
        console.error("Could not remove node!");
        return;
      }
      parent_node_render_node.children =
        parent_node_render_node.children.filter(
          (n) => n.node_id === node_to_remove_id,
        );
      generic_parent_game_node.children =
        generic_parent_game_node.children.filter(
          (c) => render_key(get_generic_game_node(c)) !== node_to_remove_id,
        );
      parent_node_render_node.container.removeChild(node_to_remove.container);
      delete render_graph_data.entity_node_map[node_to_remove_id];
      delete render_graph_data.entity_node_to_render_node_map[
        node_to_remove_id
      ];
    },
    add_node_to_graph(
      entity_node_to_render_node_map: RenderGraphData["entity_node_to_render_node_map"],
      entity_node_map: RenderGraphData["entity_node_map"],
      parent: Node,
      game_node_to_add: GameNodeKind,
      resource_manager: ResourceManager,
      create_display_object_cb: typeof create_display_object,
    ) {
      const generic_game_node = get_generic_game_node(game_node_to_add);
      const new_node_container = markRaw(
        create_display_object_cb(game_node_to_add, resource_manager),
      );
      const parent_container = parent.container;
      const new_node: Node = {
        children: [],
        parent,
        container: new_node_container,
        node_id: render_key(generic_game_node),
      };
      parent.children.push(new_node);
      console.log(parent_container, new_node_container);
      parent_container.addChild(new_node_container);
      entity_node_to_render_node_map[new_node.node_id] = new_node;
      entity_node_map[new_node.node_id] = game_node_to_add;
      return new_node;
    },
    render_key_from_game_node(game_node: GameNodeKind) {
      return render_key(get_generic_game_node(game_node));
    },
    generate_render_graph(
      entity_node_to_render_node_map: RenderGraphData["entity_node_to_render_node_map"],
      entity_node_map: RenderGraphData["entity_node_map"],
      parent: Node,
      game_node: GameNodeKind,
      resource_manager: ResourceManager,
      create_display_object_cb: typeof create_display_object,
    ) {
      const generic_game_node = get_generic_game_node(game_node);
      for (const game_node_child of generic_game_node.children) {
        const new_node = this.add_node_to_graph(
          entity_node_to_render_node_map,
          entity_node_map,
          parent,
          game_node_child,
          resource_manager,
          create_display_object_cb,
        );
        this.generate_render_graph(
          entity_node_to_render_node_map,
          entity_node_map,
          new_node,
          game_node_child,
          resource_manager,
          create_display_object_cb,
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
