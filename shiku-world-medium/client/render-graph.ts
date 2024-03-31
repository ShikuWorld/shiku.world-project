import { Container, Text } from "pixi.js-legacy";
import { Scene } from "@/editor/blueprints/Scene";
import { get_generic_game_node } from "@/editor/stores/resources";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { match, P } from "ts-pattern";
import { ResourceManager } from "@/client/resources";
import { EntityUpdate } from "@/editor/blueprints/EntityUpdate";
import { Entity } from "@/editor/blueprints/Entity";
import { Node2D } from "@/editor/blueprints/Node2D";

export interface Node {
  node_id: string | Entity;
  parent: Node | null;
  children: Node[];
  container: Container;
}

export class RenderGraph {
  render_root: Node;
  entity_node_to_render_node_map: Map<string | Entity, Node> = new Map();
  entity_node_map: Map<string | Entity, GameNodeKind> = new Map();
  scene: Scene | null = null;

  constructor() {
    this.render_root = {
      node_id: 0,
      children: [],
      container: new Container(),
      parent: null,
    };
  }

  render_graph_from_scene(scene: Scene, resource_manager: ResourceManager) {
    this.scene = scene;
    const game_node_root = get_generic_game_node(scene.root_node);
    this.render_root = {
      node_id: game_node_root.id,
      children: [],
      container: this.create_display_object(scene.root_node, resource_manager),
      parent: null,
    };
    this.entity_node_to_render_node_map.set(
      game_node_root.entity_id || game_node_root.id,
      this.render_root,
    );
    this.entity_node_map.set(
      game_node_root.entity_id || game_node_root.id,
      scene.root_node,
    );
    this.generate_render_graph(
      this.render_root,
      scene.root_node,
      resource_manager,
    );
  }

  apply_node_update(update: EntityUpdate, resource_manager: ResourceManager) {
    const node = this.entity_node_map.get(update.id);
    const render_node = this.entity_node_to_render_node_map.get(update.id);
    if (!node || !render_node) {
      console.error("Could not update game node!");
      return;
    }
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
        if (this.get_gid(game_node) === gid) {
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
  }

  generate_render_graph(
    parent: Node,
    game_node: GameNodeKind,
    resource_manager: ResourceManager,
  ) {
    const generic_game_node = get_generic_game_node(game_node);
    for (const game_node_child of generic_game_node.children) {
      const generic_game_node_child = get_generic_game_node(game_node_child);
      const new_node = {
        children: [],
        parent,
        container: this.create_display_object(
          game_node_child,
          resource_manager,
        ),
        scene_node: game_node,
        node_id:
          generic_game_node_child.entity_id || generic_game_node_child.id,
      };
      parent.children.push(new_node);
      parent.container.addChild(new_node.container);
      this.entity_node_to_render_node_map.set(new_node.node_id, new_node);
      this.entity_node_map.set(new_node.node_id, game_node_child);
      this.generate_render_graph(new_node, game_node_child, resource_manager);
    }
  }

  update_node(
    entity_id: Entity,
    node: GameNodeKind,
    resource_manager: ResourceManager,
  ) {
    const render_node = this.entity_node_to_render_node_map.get(entity_id);
    const prev_node = this.entity_node_map.get(entity_id);
    if (!render_node || !prev_node) {
      console.error(`Could not update ${entity_id}, was not present in graph!`);
      return;
    }
    const container = render_node.container;

    match(node)
      .with({ Instance: P.select() }, () => {
        console.error("No instances can be updated!");
      })
      .with({ Node2D: P.select() }, (game_node) => {
        container.x = game_node.data.transform.position[0];
        container.y = game_node.data.transform.position[1];
        container.rotation = game_node.data.transform.rotation;
        match(game_node.data.kind)
          .with({ Node2D: P.select() }, () => {})
          .with({ Render: P.select() }, (render) => {
            match(render.kind)
              .with({ Sprite: P.select() }, (gid) => {
                if (this.get_gid(prev_node) === gid) {
                  return;
                }
                const graphics = resource_manager.get_graphics_data_by_gid(gid);
                container.removeChildAt(0);
                container.addChildAt(
                  resource_manager.get_sprite_from_graphics(graphics),
                  0,
                );
              })
              .with({ AnimatedSprite: P.select() }, (gid) =>
                console.log(
                  `AnimatedSprite update not implemented, gid: ${gid}`,
                ),
              )
              .exhaustive();
          })
          .with({ RigidBody: P.select() }, (rigid_body) => {
            console.log("rb", rigid_body);
          })
          .with({ Collider: P.select() }, (collider) => {
            console.log("coll", collider);
          })
          .exhaustive();
      })
      .exhaustive();
  }

  get_gid(game_node: GameNodeKind): number | undefined {
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

  create_display_object(
    node: GameNodeKind,
    resource_manager: ResourceManager,
  ): Container {
    const container = new Container();
    match(node)
      .with({ Instance: P.select() }, () => {
        console.error("No instances can be displayed!");
      })
      .with({ Node2D: P.select() }, (game_node) => {
        container.x = game_node.data.transform.position[0];
        container.y = game_node.data.transform.position[1];
        container.rotation = game_node.data.transform.rotation;
        console.log(game_node);
        match(game_node.data.kind)
          .with({ Node2D: P.select() }, () => {
            //container.addChild(new Text(game_node.name, { fill: "white" }));
          })
          .with({ Render: P.select() }, (render) => {
            const display_object = match(render.kind)
              .with({ Sprite: P.select() }, (gid) => {
                const graphics = resource_manager.get_graphics_data_by_gid(gid);
                return resource_manager.get_sprite_from_graphics(graphics);
              })
              .with(
                { AnimatedSprite: P.select() },
                (gid) =>
                  new Text(`Animated Sprite not implemented. gid: ${gid}`, {
                    fill: "red",
                  }),
              )
              .exhaustive();
            container.addChild(display_object);
          })
          .with({ RigidBody: P.select() }, (rigid_body) => {
            console.log("rb", rigid_body);
          })
          .with({ Collider: P.select() }, (collider) => {
            console.log("coll", collider);
          })
          .exhaustive();
      })
      .exhaustive();
    return container;
  }
}
