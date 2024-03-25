import { Container, Text } from "pixi.js-legacy";
import { Scene } from "@/editor/blueprints/Scene";
import { get_generic_game_node } from "@/editor/stores/resources";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { match, P } from "ts-pattern";
import { ResourceManager } from "@/client/resources";

export interface Node {
  node_id: string;
  parent: Node | null;
  children: Node[];
  container: Container;
}

export class RenderGraph {
  render_root: Node;
  scene_node_to_render_node_map: Map<string, Node> = new Map();

  constructor() {
    this.render_root = {
      node_id: "",
      children: [],
      container: new Container(),
      parent: null,
    };
  }

  render_graph_from_scene(scene: Scene, resource_manager: ResourceManager) {
    const game_node_root = get_generic_game_node(scene.root_node);
    this.render_root = {
      node_id: game_node_root.id,
      children: [],
      container: this.create_display_object(scene.root_node, resource_manager),
      parent: null,
    };
    this.scene_node_to_render_node_map.set(game_node_root.id, this.render_root);
    this.generate_render_graph(
      this.render_root,
      scene.root_node,
      resource_manager,
    );
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
        node_id: generic_game_node_child.id,
      };
      parent.children.push(new_node);
      parent.container.addChild(new_node.container);
      this.scene_node_to_render_node_map.set(new_node.node_id, new_node);
      this.generate_render_graph(new_node, game_node_child, resource_manager);
    }
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
          .with("Node2D", () => {
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
