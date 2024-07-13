const components = import.meta.glob("./components/*.vue");
const gameNodes = import.meta.glob("./editor/game_nodes/*.vue");

export function get_gui_component(componentName: string) {
  const path = `./components/${componentName}.vue`;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return components[path]() as any;
}

export function get_game_node(node_type: string) {
  const path = `./editor/game_nodes/${node_type}.vue`;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return gameNodes[path]() as any;
}
