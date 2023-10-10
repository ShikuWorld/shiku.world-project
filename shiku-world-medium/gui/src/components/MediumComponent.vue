<template>
  <component
    :is="get_dynamic_component(component_config.name)"
    v-bind:name="component_config.name"
    v-bind:config="component_config.config"
    v-bind:input="compute_input_values(component_config.input, context)"
    v-bind:context="context"
  ></component>
</template>

<style lang="css"></style>

<script lang="ts" setup>
import { ComponentConfig, DataContext, use_layout_functions } from "@/ui";
import { MainMenu } from "@/components/MainMenu.vue";

const { compute_input_values, get_dynamic_component } = use_layout_functions();

defineProps<{ context: DataContext; component_config: ComponentConfig }>();
</script>
