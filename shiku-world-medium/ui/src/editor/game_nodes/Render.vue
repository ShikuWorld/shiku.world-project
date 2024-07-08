<template>
  <v-label class="form-label">Render</v-label>
  <v-select
    label="Render type"
    :hide-details="true"
    :items="render_options"
    :model-value="render_kind"
    @update:model-value="(new_value) => update_render_type(new_value)"
  ></v-select>
  <v-label class="form-label" v-if="render_kind == 'Sprite'">GID</v-label>
  <v-select
    v-if="render_kind == 'Sprite'"
    :hide-details="true"
    density="compact"
    :items="tileset_options"
    :item-title="'file_name'"
    :item-value="'path'"
    :model-value="sprite_tileset_path"
    @update:model-value="
      (new_value) => update_sprite_tileset_resource(new_value)
    "
  ></v-select>
  <v-text-field
    type="number"
    :hide-details="true"
    density="compact"
    :model-value="gid"
    @update:model-value="(new_value) => update_gid(new_value)"
  ></v-text-field>
  <v-label class="form-label" v-if="render_kind == 'AnimatedSprite'"
    >Character Animation</v-label
  >
  <v-select
    v-if="render_kind == 'AnimatedSprite'"
    :hide-details="true"
    density="compact"
    :items="character_animations"
    :item-title="'file_name'"
    :item-value="'path'"
    :model-value="character_animation_resource_path"
    @update:model-value="
      (new_value) => update_animated_sprite_resource(new_value)
    "
  ></v-select>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { RenderKind } from "@/editor/blueprints/RenderKind";
import { KeysOfUnion } from "@/editor/utils";
import { Render } from "@/editor/blueprints/Render";
import { match, P } from "ts-pattern";
import { EntityUpdateKind } from "@/editor/blueprints/EntityUpdateKind";
import { use_resources_store } from "@/editor/stores/resources";
import { storeToRefs } from "pinia";
import { use_editor_store } from "@/editor/stores/editor";

const { get_module } = use_resources_store();
const { selected_module_id } = storeToRefs(use_editor_store());

const props = defineProps<{ data: Render; is_instance: boolean }>();
const { data } = toRefs(props);
const render_options: Array<KeysOfUnion<RenderKind>> = [
  "AnimatedSprite",
  "Sprite",
];
const emit = defineEmits<{
  (e: "entityUpdate", data: EntityUpdateKind): void;
}>();

const character_animations = computed(() => {
  const module = get_module(selected_module_id.value);
  if (module) {
    return module.resources.filter((r) => r.kind === "CharacterAnimation");
  }
  return [];
});

const tileset_options = computed(() => {
  const module = get_module(selected_module_id.value);
  if (module) {
    return module.resources.filter((r) => r.kind === "Tileset");
  }
  return [];
});

const render_kind = computed(() => {
  return Object.keys(data.value.kind)[0] as KeysOfUnion<RenderKind>;
});

const character_animation_resource_path = computed(() => {
  if (Object.keys(data.value.kind)[0] === "AnimatedSprite") {
    return Object.values(data.value.kind)[0][0];
  }
  return null;
});

const sprite_tileset_path = computed(() => {
  if (Object.keys(data.value.kind)[0] === "Sprite") {
    return Object.values(data.value.kind)[0][0];
  }
  return null;
});

const gid = computed(() => {
  return match(data.value.kind)
    .with({ Sprite: P.select() }, ([_, gid]) => gid)
    .with({ AnimatedSprite: P.select() }, ([_, gid]) => gid)
    .exhaustive();
});

function update_render_type(kind: KeysOfUnion<RenderKind>) {
  if (kind === "Sprite") {
    emit("entityUpdate", { RenderKind: { Sprite: ["", gid.value] } });
  } else if (
    kind === "AnimatedSprite" &&
    character_animations.value.length > 0
  ) {
    emit("entityUpdate", {
      RenderKind: {
        AnimatedSprite: [character_animations.value[0].path, gid.value],
      },
    });
  }
}

function update_gid(gid: string) {
  emit("entityUpdate", { Gid: Number(gid) });
}

function update_sprite_tileset_resource(resource_path: string) {
  emit("entityUpdate", { SpriteTilesetResource: resource_path });
}

function update_animated_sprite_resource(resource_path: string) {
  emit("entityUpdate", { AnimatedSpriteResource: resource_path });
}
</script>
<style></style>
