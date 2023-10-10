<template>
  <v-list>
    <v-list-item
      v-for="stat in config.stats"
      :key="stat.label"
      :title="stat.label"
    >
      <template v-slot:append>
        <v-badge
          color="primary"
          :content="compute_input_value(stat.number, context)"
          inline
        ></v-badge>
      </template>
    </v-list-item>
  </v-list>
</template>

<script lang="ts" setup>
import { DataContext, ExtractionObject, use_layout_functions } from "@/ui";

export interface NumberStat {
  label: string;
  number: ExtractionObject;
}

export interface NumberStats {
  name: "NumberStats";
  config: { stats: NumberStat[] };
  context: DataContext;
}

const { compute_input_value } = use_layout_functions();

defineProps<NumberStats>();
</script>
