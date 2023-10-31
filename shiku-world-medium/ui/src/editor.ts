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

class Node extends ClassicPreset.Node {
  width = 180;
  height = 120;
}
class Connection<N extends Node> extends ClassicPreset.Connection<N, N> {}

type Schemes = GetSchemes<Node, Connection<Node>>;
type AreaExtra = VueArea2D<Schemes>;

export class ModuleEditor {
  socket: ClassicPreset.Socket;
  editor: NodeEditor<Schemes>;
  connection: ConnectionPlugin<Schemes, AreaExtra>;
  render: VuePlugin<Schemes, AreaExtra>;
  area: AreaPlugin<Schemes, AreaExtra> | undefined;
  arrange: AutoArrangePlugin<Schemes>;
  applier: ArrangeAppliers.TransitionApplier<Schemes, never> | undefined;
  constructor() {
    this.socket = new ClassicPreset.Socket("socket");
    this.area = undefined;
    this.editor = new NodeEditor<Schemes>();
    this.connection = new ConnectionPlugin<Schemes, AreaExtra>();
    this.render = new VuePlugin<Schemes, AreaExtra>();
    this.arrange = new AutoArrangePlugin<Schemes>();
  }

  async addModuleNode(module_blueprint: Module) {
    const node = new Node(module_blueprint.name);
    for (const entry_point in module_blueprint.insert_points) {
      node.addInput(entry_point, new ClassicPreset.Input(this.socket));
    }
    for (const exit_point in module_blueprint.exit_points) {
      node.addOutput(exit_point, new ClassicPreset.Input(this.socket));
    }
    await this.editor.addNode(node);
  }

  async layout() {
    console.log(this.area, this.applier);
    if (!this.area || !this.applier) {
      return;
    }
    await this.arrange.layout({ applier: this.applier });
    AreaExtensions.zoomAt(this.area, this.editor.getNodes());
  }

  async initRender(container: HTMLElement) {
    this.area = new AreaPlugin<Schemes, AreaExtra>(container);
    AreaExtensions.selectableNodes(this.area, AreaExtensions.selector(), {
      accumulating: AreaExtensions.accumulateOnCtrl(),
    });
    const editor = this.editor;
    const area = this.area;
    this.render.addPreset(Presets.classic.setup());

    this.connection.addPreset(ConnectionPresets.classic.setup());

    this.applier = new ArrangeAppliers.TransitionApplier<Schemes, never>({
      duration: 500,
      timingFunction: (t) => t,
      async onTick() {
        await AreaExtensions.zoomAt(area, editor.getNodes());
      },
    });

    this.arrange.addPreset(ArrangePresets.classic.setup());

    this.editor.use(area);
    area.use(this.connection);
    area.use(this.render);
    area.use(this.arrange);

    AreaExtensions.simpleNodesOrder(area);

    await this.layout();

    AreaExtensions.zoomAt(area, this.editor.getNodes());
  }

  destroy() {
    if (this.area) {
      this.area.destroy();
    }
  }
}
