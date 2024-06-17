<template>
  <div class="animations" style="width: 500px; height: 500px">
    <VueFlow class="flow" :nodes="nodes" :edges="edges">
      <Panel position="top-left">
        <button @click="addNode">Add node</button>
        <button @click="deleteSelected">Delete Selected</button>
      </Panel>
    </VueFlow>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, toRefs, onMounted } from "vue";
import { VueFlow, Panel, useVueFlow } from "@vue-flow/core";
import type { Node, Edge } from "@vue-flow/core";
import { CharacterAnimation } from "@/editor/blueprints/CharacterAnimation";

const props = defineProps<{
  characterAnimation: CharacterAnimation;
}>();

const { characterAnimation } = toRefs(props);

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

const last_transition_id = computed(() => {
  return (
    Math.max(
      ...Object.keys(characterAnimation.value.transitions).map((key) =>
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

const {
  addNodes,
  addEdges,
  removeNodes,
  removeEdges,
  fitView,
  findNode,
  onInit,
} = useVueFlow();

onInit((instance) => {
  // `instance` is the same type as the return of `useVueFlow` (VueFlowStore)

  setInterval(() => {
    console.log(instance.viewportRef.value);
    instance.$reset();
    fitView();
  }, 2000);

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
  characterAnimation.value.states[newNodeId] = {
    name: "New State",
    frames: [],
  };
  addNodes([
    {
      id: `${newNodeId}`,
      position: { x: 100, y: 100 },
      data: { label: "New State" },
    },
  ]);
}

function addEdge() {
  console.log("uh?");
  if (selectedNode.value) {
    const new_transition_id = last_transition_id.value + 1;
    if (!characterAnimation.value.transitions[new_transition_id]) {
      characterAnimation.value.transitions[new_transition_id] = {};
    }
    characterAnimation.value.transitions[new_transition_id][
      parseInt(selectedNode.value.id)
    ] = parseInt(selectedNode.value.id);
    addEdges([
      {
        id: transition_key(
          new_transition_id,
          selectedNode.value.id,
          selectedNode.value.id,
        ),
        source: selectedNode.value.id,
        target: selectedNode.value.id,
      },
    ]);
  }
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
  width: 700px;
  height: 700px;
  position: absolute;
}
.flow {
  pointer-events: all;
  width: 700px;
  height: 700px;
  position: absolute;
}
</style>
