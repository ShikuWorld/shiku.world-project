import {
  keymap,
  highlightSpecialChars,
  drawSelection,
  highlightActiveLine,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  lineNumbers,
  highlightActiveLineGutter,
} from "@codemirror/view";
import { Extension, EditorState } from "@codemirror/state";
import {
  defaultHighlightStyle,
  syntaxHighlighting,
  indentOnInput,
  bracketMatching,
  foldGutter,
  foldKeymap,
} from "@codemirror/language";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import {
  autocompletion,
  completionKeymap,
  closeBrackets,
  closeBracketsKeymap,
  CompletionContext,
  CompletionResult,
} from "@codemirror/autocomplete";
import { lintKeymap } from "@codemirror/lint";

function shikuCompletions(context: CompletionContext): CompletionResult | null {
  const word = context.matchBefore(/shiku::(.*)/);
  console.log(word);
  if (!word || (word.from == word.to && !context.explicit)) return null;

  const options = [
    {
      label: "shiku::utils::random_num_in_range",
      type: "function",
      info: "Generate a random number within a range",
      detail: "(start: number, length: number) -> number",
    },
    {
      label: "shiku::physics::is_grounded",
      type: "function",
      info: "Check if an entity is grounded",
      detail: "(entity: Entity) -> boolean",
    },
    {
      label: "shiku::physics::get_rigid_body_handle",
      type: "function",
      info: "Get the rigid body handle for an entity",
      detail: "(entity: Entity) -> RigidBodyHandle",
    },
    {
      label:
        "shiku::physics::resolve_kinematic_body_collision_impulses_automatic",
      type: "function",
      info: "Resolves all collision impulses for this frame and applies the correct desired translation changes",
      detail: "() -> void",
    },
    {
      label: "shiku::physics::set_entity_desired_translation",
      type: "function",
      info: "Set the desired translation for an entity",
      detail: "(entity: Entity, x: number, y: number) -> void",
    },
    {
      label: "shiku::physics::add_entity_desired_translation",
      type: "function",
      info: "Add the desired translation for an entity",
      detail: "(entity: Entity, x: number, y: number) -> void",
    },
    {
      label: "shiku::physics::apply_entity_friction_x",
      type: "function",
      info: "Apply friction to desired translation in x direction",
      detail: "(entity: Entity, friction_x: number) -> void",
    },
    {
      label: "shiku::physics::apply_entity_linear_dampening",
      type: "function",
      info: "Apply linear dampening to desired translation",
      detail: "(entity: Entity, dampening: number) -> void",
    },
    {
      label: "shiku::physics::add_force_to_rigid_body",
      type: "function",
      info: "Add force to a rigid body",
      detail: "(entity: Entity, force_x: number, force_y: number) -> void",
    },
    {
      label: "shiku::physics::apply_impulse_to_rigid_body",
      type: "function",
      info: "Apply impulse to a rigid body",
      detail: "(entity: Entity, force_x: number, force_y: number) -> void",
    },
    {
      label: "shiku::nodes::get_child_animation_entity",
      type: "function",
      info: "Get the child animation entity",
      detail: "(entity: Entity) -> Entity",
    },
    {
      label: "shiku::nodes::spawn_entity_from_scene",
      type: "function",
      info: "Spawn an entity from a scene",
      detail:
        "(parent_entity: Entity, source: string, x: number, y: number) -> Entity",
    },
    {
      label: "shiku::nodes::get_first_child_entity_by_tag",
      type: "function",
      info: "Get child entity with specific tag, returns void if none exists",
      detail: "(entity: Entity, tag: string) -> Entity | void",
    },
    {
      label: "shiku::nodes::get_first_child_entity_by_tag",
      type: "function",
      info: "Get child entity with specific tag, returns void if none exists",
      detail: "(entity: Entity, tag: string) -> Entity | void",
    },
    {
      label: "shiku::nodes::set_text",
      type: "function",
      info: "Set text on text render node",
      detail: "(entity: Entity, text: string) -> void",
    },
    {
      label: "shiku::animation::get_state",
      type: "function",
      info: "Get the current state of an animation",
      detail: "(entity: Entity) -> number",
    },
    {
      label: "shiku::animation::go_to_state",
      type: "function",
      info: "Set the animation state",
      detail: "(entity: Entity, state_id: number) -> void",
    },
    {
      label: "shiku::animation::get_progress",
      type: "function",
      info: "Get the progress of the current animation",
      detail: "(entity: Entity) -> number",
    },
    {
      label: "shiku::animation::set_direction",
      type: "function",
      info: "Set the direction of the character animation",
      detail: "(entity: Entity, direction: CharacterDirection) -> void",
    },
    {
      label: "shiku::actors::is_key_down",
      type: "function",
      info: "Check if a key is pressed for an actor",
      detail: "(actor_id: ActorId, key: string) -> boolean",
    },
    {
      label: "shiku::actors::is_admin",
      type: "function",
      info: "Check if actor is admin",
      detail: "(actor_id: ActorId) -> boolean",
    },
    {
      label: "shiku::actors::camera_follow_entity",
      type: "function",
      info: "Make actor camera follow entity",
      detail: "(actor_id: ActorId, entity_id: EntityId)",
    },
    {
      label: "shiku::actors::camera_set_free",
      type: "function",
      info: "Make actor not follow any entity",
      detail: "(actor_id: ActorId)",
    },
    {
      label: "shiku::actors::get_actor_display_name",
      type: "function",
      info: "Get the display name of an actor",
      detail: "(actor_id: ActorId) -> string",
    },
    {
      label: "shiku::actors::get_actor_provider_id",
      type: "function",
      info: "Get the provider ID of an actor",
      detail: "(actor_id: ActorId) -> string",
    },
    {
      label: "shiku::actors::get_active_actors",
      type: "function",
      info: "Get a list of active actors",
      detail: "() -> Array<ActorId>",
    },
  ];

  return {
    from: word.from,
    options: options.filter((option) => option.label.startsWith(word.text)),
  };
}

export const basicSetup: Extension = (() => [
  lineNumbers(),
  highlightActiveLineGutter(),
  highlightSpecialChars(),
  history(),
  foldGutter(),
  drawSelection(),
  dropCursor(),
  EditorState.allowMultipleSelections.of(true),
  indentOnInput(),
  syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
  bracketMatching(),
  closeBrackets(),
  autocompletion({ override: [shikuCompletions] }),
  rectangularSelection(),
  crosshairCursor(),
  highlightActiveLine(),
  highlightSelectionMatches(),
  keymap.of([
    ...closeBracketsKeymap,
    ...defaultKeymap,
    ...searchKeymap,
    ...historyKeymap,
    ...foldKeymap,
    ...completionKeymap,
    ...lintKeymap,
  ]),
])();
