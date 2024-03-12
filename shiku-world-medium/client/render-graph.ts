import { Container, Text } from "pixi.js-legacy";
import { Scene } from "@/editor/blueprints/Scene";
import { get_generic_game_node } from "@/editor/stores/resources";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";

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

  render_graph_from_scene(scene: Scene) {
    const game_node_root = get_generic_game_node(scene.root_node);
    this.render_root = {
      node_id: game_node_root.id,
      children: [],
      container: this.create_display_object(scene.root_node),
      parent: null,
    };
    this.scene_node_to_render_node_map.set(game_node_root.id, this.render_root);
    this.generate_render_graph(this.render_root, scene.root_node);
  }

  generate_render_graph(parent: Node, game_node: GameNodeKind) {
    const generic_game_node = get_generic_game_node(game_node);
    for (const game_node_child of generic_game_node.children) {
      const generic_game_node_child = get_generic_game_node(game_node_child);
      const new_node = {
        children: [],
        parent,
        container: this.create_display_object(game_node_child),
        node_id: generic_game_node_child.id,
      };
      parent.children.push(new_node);
      parent.container.addChild(new_node.container);
      this.scene_node_to_render_node_map.set(new_node.node_id, new_node);
      this.generate_render_graph(new_node, game_node_child);
    }
  }

  create_display_object(node: GameNodeKind): Container {
    const container = new Container();
    const game_node = get_generic_game_node(node);
    container.addChild(new Text(game_node.name, { fill: "white" }));
    return container;
  }
}
