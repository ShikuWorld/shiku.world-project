import { defineStore } from "pinia";
import { Entity } from "@/editor/blueprints/Entity";
import { GameNodeKind } from "@/editor/blueprints/GameNodeKind";
import { Scene } from "@/editor/blueprints/Scene";
import { Container } from "pixi.js";
import { ResourceManager } from "@/client/resources";
import {
  get_generic_game_node,
  use_resources_store,
} from "@/editor/stores/resources";
import { EntityUpdate } from "@/editor/blueprints/EntityUpdate";
import { match, P } from "ts-pattern";
import { Node2D } from "@/editor/blueprints/Node2D";
import { GameNode } from "@/editor/blueprints/GameNode";
import { markRaw, reactive, toRefs } from "vue";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { RENDER_SCALE } from "@/shared/index";
import { ScopeCacheValue } from "@/editor/blueprints/ScopeCacheValue";
import { KinematicCharacterControllerProps } from "@/editor/blueprints/KinematicCharacterControllerProps";

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
        scope_cache: {
          [game_node_id: string]: { [scope_key: string]: ScopeCacheValue };
        };
      };
    };
  };
  show_entity_colliders: boolean;
  blueprint_render: {
    render_graph_data?: RenderGraphData;
    scene_resource_path: string;
    is_pinned: boolean;
    module_id: string;
  } | null;
}

export const use_game_instances_store = defineStore("game-instances", () => {
  const { get_or_load_scene } = use_resources_store();
  const { scene_map } = toRefs(use_resources_store());

  const state: GameInstancesStore = reactive({
    game_instance_data_map: {},
    blueprint_render: null,
    show_entity_colliders: false,
  });

  const actions = {
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

      const render_graph_data = this.render_graph_from_scene(
        scene,
        resource_module,
      );
      state.blueprint_render = {
        scene_resource_path,
        render_graph_data,
        is_pinned,
        module_id,
      };
      if (!state.blueprint_render.render_graph_data?.render_root.container) {
        return;
      }
      window.medium.set_blueprint_renderer(
        state.blueprint_render as GameInstancesStore["blueprint_render"],
      );
    },
    add_game_instance_data(
      instance_id: string,
      world_id: string,
      create_container: () => Container,
    ) {
      if (!state.game_instance_data_map[instance_id]) {
        state.game_instance_data_map[instance_id] = {};
      }
      state.game_instance_data_map[instance_id][world_id] = {
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
        scope_cache: {},
        instance_scene: null,
      };
    },
    remove_game_instance(instance_id: string, world_id: string) {
      if (!state.game_instance_data_map[instance_id]) {
        return;
      }
      delete state.game_instance_data_map[instance_id][world_id];
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
        !state.game_instance_data_map[instance_id] ||
        !state.game_instance_data_map[instance_id][world_id]
      ) {
        console.error(
          `Could not get render graph of ${instance_id} ${world_id}`,
        );
        return null;
      }
      return state.game_instance_data_map[instance_id][world_id];
    },
    get_render_graph_data(instance_id: string, world_id: string) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      return game_instance_data?.render_graph_data;
    },
    game_instance_exists(instance_id: string, world_id: string): boolean {
      return (
        !!state.game_instance_data_map[instance_id] &&
        !!state.game_instance_data_map[instance_id][world_id]
      );
    },
    render_graph_from_scene_for_instance(
      instance_id: string,
      world_id: string,
      instance_scene: Scene,
      resource_manager: ResourceManager,
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
      );
    },
    render_graph_from_scene(
      scene: Scene,
      resource_manager: ResourceManager,
    ): RenderGraphData {
      const game_node_root = get_generic_game_node(scene.root_node);
      const render_graph_data: RenderGraphData = {
        render_root: {
          node_id: render_key(game_node_root),
          children: [],
          container: markRaw(
            window.medium.create_display_object(
              scene.root_node,
              resource_manager,
              state.show_entity_colliders,
            ),
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

      // These might be set before the graph is even rendered
      const update_applied = match(update.kind)
        .with({ UpdateScriptScope: P.select() }, ([scope_key, scope_value]) => {
          if (!game_instance_data.scope_cache[update.id]) {
            game_instance_data.scope_cache[update.id] = {};
          }
          game_instance_data.scope_cache[update.id][scope_key] = scope_value;
          return true;
        })
        .with({ SetScriptScope: P.select() }, (scope_cache_update) => {
          game_instance_data.scope_cache[update.id] = scope_cache_update;
          return true;
        })
        .otherwise(() => false);

      if (!update_applied) {
        this.apply_entity_update(render_graph_data, update, resource_manager);
      }
    },
    update_render_positions(instance_id: string, world_id: string) {
      const game_instance_data = this.get_game_instance_data(
        instance_id,
        world_id,
      );
      if (!game_instance_data) {
        return;
      }
      const render_graph_data = game_instance_data.render_graph_data;
      for (const [id, render_node] of Object.entries(
        render_graph_data.entity_node_to_render_node_map,
      )) {
        const node = render_graph_data.entity_node_map[id];
        if (node) {
          this.update_render_position(render_node, node);
        }
      }
    },
    update_render_position(render_node: Node, game_node: GameNodeKind) {
      const node_2d = get_generic_game_node(game_node).data as Node2D;
      if (node_2d.transform) {
        render_node.container.position.x = Math.round(
          node_2d.transform.position[0] * RENDER_SCALE,
        );
        render_node.container.position.y = Math.round(
          node_2d.transform.position[1] * RENDER_SCALE,
        );
        render_node.container.rotation = node_2d.transform.rotation;
      }
    },
    toggle_entity_colliders() {
      state.show_entity_colliders = !state.show_entity_colliders;
      for (const game_instance_data of Object.values(
        state.game_instance_data_map,
      )) {
        for (const render_graph_data of Object.values(game_instance_data)) {
          for (const node_id in render_graph_data.render_graph_data
            .entity_node_map) {
            const render_node =
              render_graph_data.render_graph_data
                .entity_node_to_render_node_map[node_id];
            const game_node =
              render_graph_data.render_graph_data.entity_node_map[node_id];
            if ("Collider" in game_node.Node2D.data.kind) {
              render_node.container.visible = state.show_entity_colliders;
            }
          }
        }
      }
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
        console.error(
          "Could not update game node!",
          node,
          render_node,
          update,
          render_graph_data,
        );
        return;
      }
      const game_node = Object.values(node)[0];
      match(update.kind)
        .with({ Tags: P.select() }, (tags) => {
          game_node.tags = tags;
        })
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
          rigid_body.kinematic_character_controller_props = match(
            rigid_body_type,
          )
            .with("Dynamic", "Fixed", () => null)
            .with(
              "KinematicPositionBased",
              "KinematicVelocityBased",
              () =>
                ({
                  normal_nudge_factor: 0.0,
                  slide: false,
                  max_slope_climb_angle: 45.0,
                  min_slope_slide_angle: 45.0,
                  offset: 0.0,
                  snap_to_ground: null,
                  up: [0, -1],
                  autostep: null,
                }) as KinematicCharacterControllerProps,
            )
            .exhaustive();
        })
        .with({ PositionRotation: P.select() }, ([x, y, r]) => {
          if (!game_node.data.transform) {
            console.error("Tried to update Node without transform, wtf?");
            return;
          }
          const node_2d = game_node.data as Node2D;
          node_2d.transform.position = [x, y];
          node_2d.transform.rotation = r;
        })
        .with({ Gid: P.select() }, (gid) => {
          if (get_gid(game_node) === gid) {
            return;
          }
          const graphics = match((game_node.data as Node2D).kind)
            .with({ Render: { kind: P.select() } }, (render_kind) =>
              match(render_kind)
                .with({ Text: P.select() }, () => {
                  return resource_manager.get_graphics_by_id_and_tileset_path(
                    0,
                    "TRIED_TO_SET_GID_ON_TEXT_WTF?",
                  );
                })
                .with({ Sprite: P.select() }, () => {
                  const sprite_render = render_kind as {
                    Sprite: [string, number];
                  };
                  sprite_render.Sprite[1] = gid;
                  return resource_manager.get_graphics_by_id_and_tileset_path(
                    sprite_render.Sprite[1],
                    sprite_render.Sprite[0],
                  );
                })
                .with({ AnimatedSprite: P.select() }, () => {
                  const animated_sprite_node = render_kind as {
                    AnimatedSprite: [string, number];
                  };
                  animated_sprite_node.AnimatedSprite[1] = gid;
                  return resource_manager.get_graphics_by_id_and_tileset_path(
                    animated_sprite_node.AnimatedSprite[1],
                    resource_manager.character_animation_to_tileset_map[
                      animated_sprite_node.AnimatedSprite[0]
                    ],
                  );
                })
                .exhaustive(),
            )
            .run();
          render_node.container.removeChildAt(0);
          render_node.container.addChildAt(
            resource_manager.get_sprite_from_graphics(graphics),
            0,
          );
        })
        .with({ SpriteTilesetResource: P.select() }, (resource) => {
          const graphics = match((game_node.data as Node2D).kind)
            .with({ Render: { kind: P.select() } }, (render_kind) =>
              match(render_kind)
                .with({ Sprite: P.select() }, () => {
                  const sprite_render = render_kind as {
                    Sprite: [string, number];
                  };
                  sprite_render.Sprite[0] = resource;
                  return resource_manager.get_graphics_by_id_and_tileset_path(
                    sprite_render.Sprite[1],
                    sprite_render.Sprite[0],
                  );
                })
                .with({ AnimatedSprite: P.select() }, () => {
                  return resource_manager.get_graphics_by_id_and_tileset_path(
                    0,
                    "TRIED_TO_SET_SPRITE_TILESET_ON_ANIMATED_SPRITE_WTF?",
                  );
                })
                .with({ Text: P.select() }, () => {
                  return resource_manager.get_graphics_by_id_and_tileset_path(
                    0,
                    "TRIED_TO_SET_SPRITE_TILESET_ON_TEXT_WTF?",
                  );
                })
                .exhaustive(),
            )
            .run();
          render_node.container.removeChildAt(0);
          render_node.container.addChildAt(
            resource_manager.get_sprite_from_graphics(graphics),
            0,
          );
        })
        .with({ AnimatedSpriteResource: P.select() }, (resource_path) => {
          match((game_node.data as Node2D).kind)
            .with({ Render: { kind: P.select() } }, (render_kind) => {
              match(render_kind)
                .with({ Sprite: P.select() }, () => {})
                .with({ Text: P.select() }, () => {})
                .with({ AnimatedSprite: P.select() }, () => {
                  (
                    render_kind as { AnimatedSprite: [string, number] }
                  ).AnimatedSprite[0] = resource_path;
                })
                .exhaustive();
            })
            .run();
        })
        .with({ RenderKind: P.select() }, (render_kind) => {
          const node2D = game_node.data as Node2D;
          if ("Render" in node2D.kind && node2D.kind.Render.kind) {
            node2D.kind.Render.kind = render_kind;

            const container = match(render_kind)
              .with({ Sprite: P.select() }, ([tileset_path, id_in_tileset]) =>
                resource_manager.get_sprite_from_graphics(
                  resource_manager.get_graphics_by_id_and_tileset_path(
                    id_in_tileset,
                    tileset_path,
                  ),
                ),
              )
              .with(
                { AnimatedSprite: P.select() },
                ([char_anim_resource_path, id_in_tileset]) =>
                  resource_manager.get_sprite_from_graphics(
                    resource_manager.get_graphics_by_id_and_tileset_path(
                      id_in_tileset,
                      resource_manager.character_animation_to_tileset_map[
                        char_anim_resource_path
                      ],
                    ),
                  ),
              )
              .with({ Text: P.select() }, (text) =>
                resource_manager.create_bitmap_text(text),
              )
              .exhaustive();
            render_node.container.removeChildAt(0);
            render_node.container.addChildAt(container, 0);
          }
        })
        .with({ InstancePath: P.select() }, (_) => {
          console.error(
            "There should ve no InstancePath updates from the backend at this point",
          );
        })
        .with({ Collider: P.select() }, (collider) => {
          if (!game_node.data.transform) {
            console.error(
              "Tried to update rigid body without a transform, wtf?",
            );
            return;
          }
          const node_2d = game_node.data as Node2D;
          if (!("Collider" in node_2d.kind)) {
            console.error("Could not upate collider");
            return;
          }
          node_2d.kind.Collider = collider;

          const [graphics, pivot_x, pivot_y] =
            window.medium.create_collider_graphic(collider);
          render_node.container.removeChildAt(0);
          render_node.container.addChildAt(graphics, 0);
          render_node.container.x = pivot_x;
          render_node.container.y = pivot_y;
          render_node.container.visible = state.show_entity_colliders;
        })
        .with(
          { UpdateScriptScope: P.select() },
          ([_scope_key, _scope_value]) => {
            /* dealt with one level up */
          },
        )
        .with({ KinematicCharacterControllerProps: P.select() }, (props) => {
          const node_2d = game_node.data as Node2D;
          const rigid_body =
            "RigidBody" in node_2d.kind ? node_2d.kind.RigidBody : null;
          if (!rigid_body) {
            console.error("Could not upate rigid body type");
            return;
          }
          rigid_body.kinematic_character_controller_props = props;
        })
        .with({ SetScriptScope: P.select() }, (_scope_cache_update) => {
          /* dealt with one level up */
        })
        .with({ TextRender: P.select() }, (text_render) => {
          const node2D = game_node.data as Node2D;
          if ("Render" in node2D.kind && node2D.kind.Render.kind) {
            if ("Text" in node2D.kind.Render.kind) {
              node2D.kind.Render.kind.Text = text_render;
              const container =
                resource_manager.create_bitmap_text(text_render);
              render_node.container.removeChildAt(0);
              render_node.container.addChildAt(container, 0);
            }
          }
        })
        .exhaustive();
    },
    add_child_to_render_graph(
      render_graph_data: RenderGraphData,
      parent_node_id: string | number,
      node_to_insert: GameNodeKind,
      resource_manager: ResourceManager,
    ) {
      const parent_node_render_node =
        render_graph_data.entity_node_to_render_node_map[parent_node_id];
      const parent_node_game_node =
        render_graph_data.entity_node_map[parent_node_id];
      if (!parent_node_render_node || !parent_node_game_node) {
        console.error("Could not add child to node!");
        return;
      }
      const node_to_insert_generic = get_generic_game_node(node_to_insert);
      if (
        node_to_insert_generic.entity_id &&
        render_graph_data.entity_node_map[node_to_insert_generic.entity_id]
      ) {
        console.warn("Node already exists in render graph!");
        return;
      }
      const new_node = this.add_node_to_graph(
        render_graph_data.entity_node_to_render_node_map,
        render_graph_data.entity_node_map,
        parent_node_render_node,
        node_to_insert,
        resource_manager,
      );
      get_generic_game_node(parent_node_game_node).children.push(
        node_to_insert,
      );
      this.generate_render_graph(
        render_graph_data.entity_node_to_render_node_map,
        render_graph_data.entity_node_map,
        new_node,
        node_to_insert,
        resource_manager,
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
    ) {
      const generic_game_node = get_generic_game_node(game_node_to_add);
      const new_node_container = window.medium.create_display_object(
        game_node_to_add,
        resource_manager,
        state.show_entity_colliders,
      );
      const parent_container = parent.container;
      const new_node: Node = {
        children: [],
        parent,
        container: markRaw(new_node_container),
        node_id: render_key(generic_game_node),
      };
      parent.children.push(new_node);
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
    ) {
      const generic_game_node = get_generic_game_node(game_node);
      let game_node_children = generic_game_node.children;
      if ("Node2D" in game_node && "Instance" in game_node.Node2D.data.kind) {
        const scene = get_or_load_scene(
          scene_map.value,
          game_node.Node2D.data.kind.Instance,
        );
        if (!scene) {
          return;
        }
        game_node_children = [scene.root_node];
      }
      for (const game_node_child of game_node_children) {
        const new_node = this.add_node_to_graph(
          entity_node_to_render_node_map,
          entity_node_map,
          parent,
          game_node_child,
          resource_manager,
        );
        this.generate_render_graph(
          entity_node_to_render_node_map,
          entity_node_map,
          new_node,
          game_node_child,
          resource_manager,
        );
      }
    },
  };

  return {
    ...toRefs(state),
    ...actions,
  };
});

function get_gid(node_2d: GameNode<Node2D>): number | undefined {
  if ("Render" in node_2d.data.kind) {
    return match(node_2d.data.kind.Render.kind)
      .with({ Sprite: P.select() }, ([_, gid]) => gid)
      .with({ Text: P.select() }, () => 0)
      .with({ AnimatedSprite: P.select() }, ([_, gid]) => gid)
      .exhaustive();
  }

  return undefined;
}

export function render_key(game_node: GameNode<unknown>) {
  return typeof game_node.entity_id === "number"
    ? game_node.entity_id
    : game_node.id;
}
