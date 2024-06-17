<template>
  <div class="animations">
    <VueFlow
      class="flow"
      :nodes="nodes"
      :edges="edges"
      @nodeClick="on_node_selected"
    >
      <Panel position="top-left">
        <button @click="addNode">Add node</button>
        <button @click="deleteSelected">Delete Selected</button>
      </Panel>
    </VueFlow>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, toRefs } from "vue";
import { VueFlow, Panel, useVueFlow, NodeMouseEvent } from "@vue-flow/core";
import type { Node, Edge } from "@vue-flow/core";
import { CharacterAnimation } from "@/editor/blueprints/CharacterAnimation";
import { use_resources_store } from "@/editor/stores/resources";

function on_node_selected(event: NodeMouseEvent) {
  emit("select_animation_node", parseInt(event.node.id));
}

const emit = defineEmits<{
  (e: "select_animation_node", node_id: number): void;
}>();

const props = defineProps<{
  characterAnimation: CharacterAnimation;
}>();

const { characterAnimation } = toRefs(props);
const { update_character_animation_server } = use_resources_store();

const nodes = computed(() => {
  return Object.entries(characterAnimation.value.states).map(
    ([key, state]) => ({
      id: key,
      label: state.name,
      position: { x: Math.random() * 400, y: Math.random() * 400 },
    }),
  );
});

const edges = computed(() => {
  return Object.entries(characterAnimation.value.transitions).flatMap(
    ([transitionId, transitions]) => {
      return Object.entries(transitions).map(([from, to]) => ({
        id: transition_key(transitionId, from, to),
        source: `${from}`,
        target: `${to}`,
      }));
    },
  );
});

const last_node_id = computed(() => {
  return (
    Math.max(
      ...Object.keys(characterAnimation.value.states).map((key) =>
        parseInt(key),
      ),
    ) || 0
  );
});

function transition_key(
  transitionId: number | string,
  from: number | string,
  to: number | string,
) {
  return `${transitionId}-${from}-${to}`;
}

const { removeNodes, removeEdges, findNode, onInit } = useVueFlow();

onInit(() => {
  console.log("flow initialized");
  const node = findNode("0");

  if (node) {
    console.log(node);
    node.position = { x: 100, y: 100 };
  }
});

const selectedNode = ref<Node | null>(null);
const selectedEdge = ref<Edge | null>(null);
const selectedNodeId = computed<number | null>(() =>
  selectedNode.value?.id ? parseInt(selectedNode.value.id) : null,
);
const selectedEdgeId = computed<number | null>(() =>
  selectedEdge.value?.id ? parseInt(selectedEdge.value.id) : null,
);

function addNode() {
  const newNodeId = last_node_id.value + 1;
  update_character_animation_server({
    ...characterAnimation.value,
    states: {
      ...characterAnimation.value.states,
      [newNodeId]: {
        name: "New State",
        frames: [],
      },
    },
  });
}

function deleteSelected() {
  if (selectedNode.value && selectedNodeId.value) {
    delete characterAnimation.value.states[selectedNodeId.value];
    removeNodes([selectedNode.value.id]);
    selectedNode.value = null;
  }
  if (selectedEdge.value && selectedEdgeId.value) {
    delete characterAnimation.value.transitions[selectedEdgeId.value];
    removeEdges([selectedEdge.value.id]);
    selectedEdge.value = null;
  }
}

watch(characterAnimation, () => {
  // TODO: Handle graph changes
});
</script>

<style>
@import "@vue-flow/core/dist/style.css";
@import "@vue-flow/core/dist/theme-default.css";

.animations {
  pointer-events: all;
  width: 1400px;
  height: 800px;
}
</style>
