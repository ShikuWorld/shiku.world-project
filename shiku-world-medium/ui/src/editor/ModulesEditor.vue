<template>
  <div class="editor" ref="rete"></div>
</template>

<script lang="ts" setup>
import { NodeEditor, GetSchemes, ClassicPreset } from "rete";
import { AreaPlugin, AreaExtensions } from "rete-area-plugin";
import {
  ConnectionPlugin,
  Presets as ConnectionPresets,
} from "rete-connection-plugin";
import { VuePlugin, Presets, VueArea2D } from "rete-vue-plugin";
import {
  AutoArrangePlugin,
  Presets as ArrangePresets,
  ArrangeAppliers,
} from "rete-auto-arrange-plugin";
import { Module } from "@/editor/blueprints/Module";
import { onMounted, onUnmounted, ref, watch } from "vue";
import { use_editor_store } from "@/editor/stores/editor";
import { storeToRefs } from "pinia";

class Node extends ClassicPreset.Node {
  width = 180;
  height = 120;
}
class Connection<N extends Node> extends ClassicPreset.Connection<N, N> {}

type Schemes = GetSchemes<Node, Connection<Node>>;
type AreaExtra = VueArea2D<Schemes>;

const socket = new ClassicPreset.Socket("socket");
const editor = new NodeEditor<Schemes>();
const connection = new ConnectionPlugin<Schemes, AreaExtra>();
const render = new VuePlugin<Schemes, AreaExtra>();
const arrange = new AutoArrangePlugin<Schemes>();
let area: AreaPlugin<Schemes, AreaExtra> | undefined = undefined;
let applier: ArrangeAppliers.TransitionApplier<Schemes, never> | undefined =
  undefined;
const rete = ref<HTMLElement>();

onMounted(async () => {
  if (!rete.value) {
    console.error("Could not find rete ref!");
    return;
  }
  area = new AreaPlugin<Schemes, AreaExtra>(rete.value);
  AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: AreaExtensions.accumulateOnCtrl(),
  });
  render.addPreset(Presets.classic.setup());

  connection.addPreset(ConnectionPresets.classic.setup());

  applier = new ArrangeAppliers.TransitionApplier<Schemes, never>({
    duration: 500,
    timingFunction: (t) => t,
    async onTick() {
      if (area) {
        await AreaExtensions.zoomAt(area, editor.getNodes());
      }
    },
  });

  arrange.addPreset(ArrangePresets.classic.setup());

  editor.use(area);
  area.use(connection);
  area.use(render);
  area.use(arrange);

  AreaExtensions.simpleNodesOrder(area);
  AreaExtensions.zoomAt(area, editor.getNodes());
  area.signal.addPipe((data) => {
    if (data.type === "nodepicked") {
      console.log(data);
    }
    return data;
  });
});

onUnmounted(() => {
  if (area) {
    area.destroy();
  }
});

const { load_modules } = use_editor_store();
const { modules } = storeToRefs(use_editor_store());

load_modules();

async function addModuleNode(module_blueprint: Module) {
  const node = new Node(module_blueprint.name);
  for (const entry_point of module_blueprint.insert_points) {
    node.addInput(
      entry_point.name,
      new ClassicPreset.Input(socket, entry_point.name),
    );
  }
  for (const exit_point of module_blueprint.exit_points) {
    node.addOutput(
      exit_point.name,
      new ClassicPreset.Input(socket, exit_point.name),
    );
  }
  await editor.addNode(node);
}
async function layout() {
  if (!area || !applier) {
    return;
  }
  await arrange.layout({ applier: applier });
  AreaExtensions.zoomAt(area, editor.getNodes());
}

addModuleNode({
  exit_points: [{ name: "LoginOut", condition_script: "" }],
  insert_points: [],
  name: "Login",
  maps: [],
  max_guests: 0,
  min_guests: 0,
  resources: [],
});

watch(modules, async () => {
  console.log("modules changes", modules.value);
  for (const module of modules.value) {
    await addModuleNode(module);
  }
  await layout();
});

watch(modules, () => {
  console.log(modules);
});

defineExpose({
  addModuleNode,
  layout,
});
</script>
