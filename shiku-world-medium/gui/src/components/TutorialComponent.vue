<template>
  <v-card>
    <v-tabs class="medium-tutorial-tabs" v-model="tab" bg-color="secondary">
      <v-tab
        v-for="tutorial in config.tutorials"
        :value="tutorial.id"
        :key="tutorial.id"
        >{{ tutorial.label }}</v-tab
      >
    </v-tabs>
    <v-card-text>
      <v-window v-model="tab">
        <v-window-item
          v-for="tutorial in config.tutorials"
          :value="tutorial.id"
          :key="tutorial.id"
        >
          <v-carousel height="400" v-if="tutorial.pics">
            <v-carousel-item
              v-for="(pic, i) in tutorial.pics"
              :key="i"
              :src="`${resource_base_url}/${pic}`"
            ></v-carousel-item>
          </v-carousel>
          <div v-if="tutorial.text">{{ tutorial.text }}</div>
        </v-window-item>
      </v-window>
    </v-card-text>
  </v-card>
</template>

<style type="scss">
.medium-tutorial-tabs {
  margin-top: 20px;
}
</style>

<script lang="ts" setup>
import { ref } from "vue";
import { use_config_store } from "@/stores/config";

export interface Tutorial {
  id: string;
  label: string;
  pics?: string[];
  text?: string;
}

export interface TutorialComponent {
  name: "TutorialComponent";
  config: { tutorials: Tutorial[] };
}

const { resource_base_url } = use_config_store();

const tab = ref(1);

const props = defineProps<TutorialComponent>();
</script>
