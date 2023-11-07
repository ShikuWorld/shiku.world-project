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
import CustomNode from "@/editor/editor/CustomNode.vue";

class Node extends ClassicPreset.Node {
  width = 180;
  height = 120;
  data: Module;
  constructor(module: Module) {
    super(module.name);
    this.data = module;
  }
}
class Connection<N extends Node> extends ClassicPreset.Connection<N, N> {}

type Schemes = GetSchemes<Node, Connection<Node>>;
type AreaExtra = VueArea2D<Schemes>;

const socket = new ClassicPreset.Socket("socket");
const editor = new NodeEditor<Schemes>();
const connection = new ConnectionPlugin<Schemes, AreaExtra>();
const render = new VuePlugin<Schemes, AreaExtra>();
const arrange = new AutoArrangePlugin<Schemes>();
const node_to_module_map: { [node_id: string]: Module } = {};
const module_to_node_map: { [module_id: string]: Node } = {};
let area: AreaPlugin<Schemes, AreaExtra> | undefined = undefined;
let applier: ArrangeAppliers.TransitionApplier<Schemes, never> | undefined =
  undefined;
const rete = ref<HTMLElement>();

const { set_selected_module_id } = use_editor_store();

onMounted(async () => {
  if (!rete.value) {
    console.error("Could not find rete ref!");
    return;
  }
  area = new AreaPlugin<Schemes, AreaExtra>(rete.value);
  AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: AreaExtensions.accumulateOnCtrl(),
  });
  render.addPreset(
    Presets.classic.setup({
      customize: {
        node() {
          return CustomNode;
        },
      },
    }),
  );

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
  area.signal.addPipe((event) => {
    if (event.type.includes("nodepicked")) {
      if (!node_to_module_map[event.data.id]) {
        console.log(event, node_to_module_map);
        console.error(`No module for this id?!`);
        return;
      }
      set_selected_module_id(node_to_module_map[event.data.id].id);
    }
    return event;
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

async function addOrUpdateNode(module_blueprint: Module) {
  let node = module_to_node_map[module_blueprint.id];
  if (node && area) {
    node.data = module_blueprint;
    await area.update("node", node.id);
    return;
  }
  node = new Node(module_blueprint);
  node_to_module_map[node.id] = module_blueprint;
  module_to_node_map[module_blueprint.id] = node;
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

addOrUpdateNode({
  id: "Login",
  exit_points: [{ name: "LoginOut", condition_script: "" }],
  insert_points: [],
  name: "Login",
  maps: [],
  max_guests: 0,
  min_guests: 0,
  close_after_full: false,
  resources: [],
});

watch(modules, async () => {
  console.log("?", modules);
  for (const module of Object.values(modules.value)) {
    await addOrUpdateNode(module);
  }
  await layout();
});

watch(modules, () => {
  console.log(modules);
});

defineExpose({
  addModuleNode: addOrUpdateNode,
  layout,
});
</script>
