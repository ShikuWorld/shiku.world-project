<template>
  <div class="medium-h-layout">
    <v-row class="medium-row">
      <v-col
        class="medium-col"
        v-for="column in config.columns"
        :cols="calc_responsive_value(column.cols, display_instance)"
      >
        <MediumComponent
          :component_config="column.component"
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
.medium-h-layout {
  height: 100%;
  display: flex;
  overflow: hidden !important;
}
</style>

<script lang="ts" setup>
import MediumComponent from "@/components/MediumComponent.vue";
import {
  ComponentConfig,
  DataContext,
  ResponsiveValue,
  use_layout_functions,
} from "@/ui";
import { useDisplay } from "vuetify";
import { ref } from "vue";

export interface HLayout {
  name: "HLayout";
  config: {
    columns: Array<{
      cols: ResponsiveValue<number>;
      component: ComponentConfig;
    }>;
  };
  context: DataContext;
}

const { calc_responsive_value } = use_layout_functions();

const display_instance = ref(useDisplay());

defineProps<HLayout>();
</script>
