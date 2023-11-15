<template>
  <div class="editor-wrap">
    <div class="editor" ref="rete"></div>
    <div class="editor-actions">
      <v-dialog width="500">
        <template v-slot:activator="{ props }">
          <v-btn :icon="mdiPlus" size="small" v-bind="props"></v-btn>
        </template>

        <template v-slot:default="{ isActive }">
          <v-card title="Lets create a new game!">
            <v-text-field label="Name" v-model="new_module_name"></v-text-field>
            <v-card-actions>
              <v-spacer></v-spacer>

              <v-btn
                text="Create Module"
                @click="
                  create_new_module(new_module_name);
                  isActive.value = false;
                "
              ></v-btn>
              <v-btn
                text="Close Dialog"
                @click="isActive.value = false"
              ></v-btn>
            </v-card-actions>
          </v-card>
        </template>
      </v-dialog>
      <v-btn :icon="mdiRefreshAuto" size="small" @click="layout"></v-btn>
    </div>
  </div>
</template>

<style lang="scss">
.editor {
  display: flex;
  height: 100%;
}
.editor-actions {
  position: absolute;
  right: 30px;
  top: 30px;
}
</style>

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
import { mdiPlus, mdiRefreshAuto } from "@mdi/js";

class Node extends ClassicPreset.Node {
  width = 180;
  height = 120;
  data: Module;
  constructor(module: Module) {
    super(module.name);
    this.data = module;
  }
}
class Connection<N extends Node> extends ClassicPreset.Connection<N, N> {
  constructor(
    source: N,
    sourceOutput: keyof N["outputs"],
    target: N,
    targetInput: keyof N["inputs"],
    public change_from_server: boolean,
  ) {
    super(source, sourceOutput, target, targetInput);
  }
}

type Schemes = GetSchemes<Node, Connection<Node>>;
type AreaExtra = VueArea2D<Schemes>;

const socket = new ClassicPreset.Socket("socket");
const editor = new NodeEditor<Schemes>();
const connection = new ConnectionPlugin<Schemes, AreaExtra>();
const render = new VuePlugin<Schemes, AreaExtra>();
const arrange = new AutoArrangePlugin<Schemes>();
const node_to_module_map: { [node_id: string]: Module } = {};
const module_to_node_map: { [module_id: string]: Node } = {};
const connection_map: { [exit_slot_name: string]: Connection<Node> } = {};
let area: AreaPlugin<Schemes, AreaExtra> | undefined = undefined;
let applier: ArrangeAppliers.TransitionApplier<Schemes, never> | undefined =
  undefined;
const rete = ref<HTMLElement>();

const { set_selected_module_id, create_module_server, save_conductor_server } =
  use_editor_store();

const new_module_name = ref<string>("");

function create_new_module(module_name: string) {
  create_module_server(module_name);
  new_module_name.value = "";
}

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
  area.signal.addPipe((context: { data: Connection<Node>; type: string }) => {
    if (context.type.includes("connectioncreate")) {
      if (context.data.change_from_server) {
        context.data.change_from_server = false;
        connection_map[context.data.sourceOutput] = context.data;
        return context;
      }
      const target_module = node_to_module_map[context.data.target];
      save_conductor_server({
        ...conductor.value,
        module_connection_map: {
          ...conductor.value.module_connection_map,
          [context.data.sourceOutput]: [
            target_module.id,
            context.data.targetInput,
          ],
        },
      });
      return false;
    }
    if (context.type.includes("connectionremove")) {
      if (context.data.change_from_server) {
        delete connection_map[context.data.sourceOutput];
        context.data.change_from_server = false;
        return context;
      }
      const new_conductor = {
        ...conductor.value,
        module_connection_map: {
          ...conductor.value.module_connection_map,
        },
      };
      delete new_conductor.module_connection_map[context.data.sourceOutput];
      save_conductor_server(new_conductor);
      return false;
    }
    if (context.type.includes("nodepicked")) {
      if (!node_to_module_map[context.data.id]) {
        console.log(context, node_to_module_map);
        console.error(`No module for this id?!`);
        return;
      }
      set_selected_module_id(node_to_module_map[context.data.id].id);
    }
    return context;
  });
});

onUnmounted(() => {
  if (area) {
    area.destroy();
  }
});

const { load_modules } = use_editor_store();
const { modules, conductor } = storeToRefs(use_editor_store());

load_modules();
function update_sockets(node: Node) {
  for (const key of Object.keys(node.inputs)) {
    if (node.data.insert_points.find((p) => p.name === key) === undefined) {
      node.removeInput(key);
    }
  }
  for (const insert_point of node.data.insert_points) {
    if (!node.inputs[insert_point.name]) {
      node.addInput(
        insert_point.name,
        new ClassicPreset.Input(socket, insert_point.name),
      );
    }
  }
  for (const key of Object.keys(node.outputs)) {
    if (node.data.exit_points.find((p) => p.name === key) === undefined) {
      node.removeOutput(key);
    }
  }
  for (const exit_point of node.data.exit_points) {
    if (!node.outputs[exit_point.name]) {
      node.addOutput(
        exit_point.name,
        new ClassicPreset.Output(socket, exit_point.name),
      );
    }
  }
}

async function update_connections() {
  if (conductor.value && modules.value) {
    for (const [
      exit_slot_name,
      [target_module_id, enter_slot_name],
    ] of Object.entries(conductor.value.module_connection_map)) {
      if (!connection_map[exit_slot_name]) {
        const source_module =
          Object.values(modules.value).find((m) =>
            m.exit_points.some((e) => e.name === exit_slot_name),
          ) || (exit_slot_name === "LoginOut" ? { id: "Login" } : undefined);
        if (!source_module) {
          console.error("Source module not found!?");
          continue;
        }
        const source_node = module_to_node_map[source_module.id];
        const target_node = module_to_node_map[target_module_id];
        if (!source_node || !target_node) {
          console.error("could not connect", source_node, target_node);
          continue;
        }
        await editor.addConnection(
          new Connection(
            source_node,
            exit_slot_name,
            target_node,
            enter_slot_name,
            true,
          ),
        );
      }
    }
    for (const [exit_slot_name, connection] of Object.entries(connection_map)) {
      if (!conductor.value.module_connection_map[exit_slot_name]) {
        connection.change_from_server = true;
        await editor.removeConnection(connection.id);
      }
    }
  }
}

async function addOrUpdateNode(module_blueprint: Module) {
  let node = module_to_node_map[module_blueprint.id];
  if (node && area) {
    node.data = module_blueprint;
    update_sockets(node);
    await area.update("node", node.id);
    return;
  }
  node = new Node(module_blueprint);
  node_to_module_map[node.id] = module_blueprint;
  module_to_node_map[module_blueprint.id] = node;
  for (const insert_point of module_blueprint.insert_points) {
    node.addInput(
      insert_point.name,
      new ClassicPreset.Input(socket, insert_point.name),
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
async function removeNode(node: Node) {
  await editor.removeNode(node.id);
  delete node_to_module_map[node.id];
  delete module_to_node_map[node.data.id];
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
  for (const module of Object.values(modules.value)) {
    await addOrUpdateNode(module);
  }
  for (const node of Object.values(module_to_node_map)) {
    if (node.data.id === "Login") {
      continue;
    }
    if (!modules.value[node.data.id]) {
      await removeNode(node);
    }
  }
  setTimeout(async () => {
    await update_connections();
    await layout();
  }, 100);
});

watch(conductor, async () => {
  await update_connections();
});

defineExpose({
  addModuleNode: addOrUpdateNode,
  layout,
});
</script>
