<template>
  <div
    class="node"
    :class="{ selected: data.selected }"
    :style="nodeStyles()"
    data-testid="node"
  >
    <div class="title" data-testid="title">{{ data.data.name }}</div>
    <!-- Outputs-->
    <div
      class="output"
      v-for="[key, output] in sortByIndex(entries(data.outputs))"
      :key="key + seed"
      :data-testid="'output-' + key"
    >
      <div class="output-title" data-testid="output-title">
        {{ output.label }}
      </div>
      <Ref
        class="output-socket"
        :emit="emit"
        :data="{
          type: 'socket',
          side: 'output',
          key: key,
          nodeId: data.id,
          payload: output.socket,
        }"
        data-testid="output-socket"
      />
    </div>
    <!-- Controls-->
    <Ref
      class="control"
      v-for="[key, control] in controls"
      :key="key + seed"
      :emit="emit"
      :data="{ type: 'control', payload: control }"
      :data-testid="'control-' + key"
    />
    <!-- Inputs-->
    <div
      class="input"
      v-for="[key, input] in sortByIndex(entries(data.inputs))"
      :key="key + seed"
      :data-testid="'input-' + key"
    >
      <Ref
        class="input-socket"
        :emit="emit"
        :data="{
          type: 'socket',
          side: 'input',
          key: key,
          nodeId: data.id,
          payload: input.socket,
        }"
        data-testid="input-socket"
      />
      <div
        class="input-title"
        v-show="!input.control || !input.showControl"
        data-testid="input-title"
      >
        {{ input.label }}
      </div>
      <Ref
        class="input-control"
        v-show="input.control && input.showControl"
        :emit="emit"
        :data="{ type: 'control', payload: input.control }"
        data-testid="input-control"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { Ref } from "rete-vue-plugin";
import { Module } from "@/editor/blueprints/Module";
import { ClassicPreset } from "rete";

type Entry<T> = { [key: string]: T };

interface DataProp {
  width?: number;
  height?: number;
  selected: boolean;
  inputs: Entry<ClassicPreset.Input<ClassicPreset.Socket>>;
  data: Module;
  id: string;
  controls: Entry<ClassicPreset.Control>;
  outputs: Entry<ClassicPreset.Output<ClassicPreset.Socket>>;
}

const props = defineProps<{
  data: DataProp;
  emit: (event: string, ...args: never[]) => void;
  seed: number;
}>();

function sortByIndex<T>(entries: [string, T & { index?: number }][]) {
  return entries.sort((a, b) => {
    const ai = a[1] && a[1].index ? a[1].index : 0;
    const bi = b[1]?.index ? b[1].index : 0;
    return ai - bi;
  });
}

function entries<T>(entry: Entry<T>) {
  return Object.entries(entry);
}

const { data, emit } = toRefs(props);

const nodeStyles = () => ({
  width: Number.isFinite(data.value.width) ? `${data.value.width}px` : "",
  height: Number.isFinite(data.value.height) ? `${data.value.height}px` : "",
});

const controls = computed(() =>
  sortByIndex(Object.entries(data.value.controls)),
);
</script>

<style lang="scss" scoped>
@use "sass:math";

$node-width: 200px;
$socket-margin: 6px;
$socket-size: 16px;

.node {
  background: black;
  border: 2px solid grey;
  border-radius: 10px;
  cursor: pointer;
  box-sizing: border-box;
  width: $node-width;
  height: auto;
  padding-bottom: 6px;
  position: relative;
  user-select: none;

  &:hover {
    background: #333;
  }

  &.selected {
    border-color: red;
  }

  .title {
    color: white;
    font-family: sans-serif;
    font-size: 18px;
    padding: 8px;
  }

  .output {
    text-align: right;
  }

  .input {
    text-align: left;
  }

  .output-socket {
    text-align: right;
    margin-right: -1px;
    display: inline-block;
  }

  .input-socket {
    text-align: left;
    margin-left: -1px;
    display: inline-block;
  }

  .input-title,
  .output-title {
    vertical-align: middle;
    color: white;
    display: inline-block;
    font-family: sans-serif;
    font-size: 14px;
    margin: $socket-margin;
    line-height: $socket-size;
  }

  .input-control {
    z-index: 1;
    width: calc(100% - #{$socket-size + 2 * $socket-margin});
    vertical-align: middle;
    display: inline-block;
  }

  .control {
    padding: $socket-margin math.div($socket-size, 2) + $socket-margin;
  }
}
</style>
