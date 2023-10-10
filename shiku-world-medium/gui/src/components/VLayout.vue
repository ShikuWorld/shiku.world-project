<template>
  <div class="medium-v-layout">
    <v-row class="medium-row" no-gutters v-for="row in config.rows">
      <v-col class="medium-col" :cols="12">
        <MediumComponent
          :component_config="row.component"
          :context="context"
        ></MediumComponent>
      </v-col>
    </v-row>
  </div>
</template>

<style type="scss">
.medium-row,
.medium-col {
  max-height: 100% !important;
}
.medium-v-layout {
  height: 100%;
  display: flex;
  overflow: hidden !important;
}
</style>

<script lang="ts" setup>
import MediumComponent from "@/components/MediumComponent.vue";
import { ComponentConfig, DataContext } from "@/ui";

export interface VLayout {
  name: "VLayout";
  config: {
    rows: Array<{
      component: ComponentConfig;
    }>;
  };
  context: DataContext;
}

defineProps<VLayout>();
</script>
