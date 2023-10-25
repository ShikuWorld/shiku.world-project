<template>
  <div class="medium-tabs">
    <div class="medium-tab-bar">
      <v-btn
        class="medium-tab-bar__tab"
        v-for="tab in config.tabs"
        :flat="true"
        :active="tab.component === current_tab_component"
        @click="switch_tab(tab.component)"
      >
        <v-icon :icon="get_icon(tab.icon)"></v-icon>
        <span
          class="medium-tab-bar__tab-text"
          v-if="is_big_screen(display_instance)"
          >{{ tab.label }}</span
        >
      </v-btn>
      <v-btn :flat="true" class="medium-tab-bar__close" @click="close()">
        <v-icon :icon="mdiCloseThick"></v-icon>
      </v-btn>
    </div>
    <v-divider></v-divider>
    <div class="medium-tab-content">
      <perfect-scrollbar class="medium-scroll">
        <MediumComponent
          v-if="current_tab_component"
          :component_config="current_tab_component"
          :context="context"
        ></MediumComponent>
      </perfect-scrollbar>
    </div>
  </div>
</template>

<style lang="scss">
.medium-scroll {
  height: 100%;
}
.medium-tabs {
  position: relative;
  pointer-events: all;
  margin: 16px !important;
  height: 100%;
  border-radius: 4px;
  overflow: hidden;
  background: rgba(var(--v-theme-surface), 0.7);
  backdrop-filter: blur(6px);
  display: flex;
  flex-flow: column;
}

.medium-tab-bar {
  flex: 0 1 auto;

  &__close {
    position: absolute !important;
    color: rgb(var(--v-theme-error)) !important;
    background: none !important;
    padding: 0 !important;
    min-width: 36px !important;
    right: 0;
    top: 0;
  }

  &__tab-text {
    margin-top: 8px;
  }

  &__tab {
    --v-btn-height: 60px !important;
    border-radius: 0 !important;
    font-size: 12px !important;
    background: none !important;

    &--active {
      --v-theme-secondary: 255, 255, 255;
    }

    .v-btn__content {
      flex-direction: column;
    }

    .v-icon {
      --v-icon-size-multiplier: 1.1 !important;
    }
  }
}

.medium-tab-content {
  flex: 1 1 auto;
  height: calc(100% - 60px);
}
</style>

<script lang="ts" setup>
import MediumComponent from "@/editor/components/MediumComponent.vue";
import { ComponentConfig, DataContext } from "@/editor/ui";
import { ref, UnwrapRef } from "vue";
import { DisplayInstance, useDisplay } from "vuetify";
import { match } from "ts-pattern";
import {
  mdiBagPersonal,
  mdiChartBar,
  mdiCog,
  mdiHelp,
  mdiCloseThick,
} from "@mdi/js";
import { use_ui_store } from "@/editor/stores/ui";

type MainMenuIcon = "mdiHelp" | "mdiCog" | "mdiBagPersonal" | "mdiChartBar";
export interface MainMenu {
  name: "MainMenu";
  config: {
    tabs: Array<{
      icon: MainMenuIcon;
      label: string;
      component: ComponentConfig;
    }>;
  };
  context: DataContext;
}

const props = defineProps<MainMenu>();

let current_tab_component = ref(props.config.tabs[0].component);

function switch_tab(component_config: ComponentConfig) {
  current_tab_component.value = component_config;
}

const display_instance = ref(useDisplay());

function is_big_screen(display_instance: UnwrapRef<DisplayInstance>): boolean {
  return display_instance.mdAndUp;
}

function get_icon(icon: MainMenuIcon): string {
  return match(icon)
    .with("mdiCog", () => mdiCog)
    .with("mdiBagPersonal", () => mdiBagPersonal)
    .with("mdiHelp", () => mdiHelp)
    .with("mdiChartBar", () => mdiChartBar)
    .exhaustive();
}

const ui = use_ui_store();

function close() {
  ui.close_menu();
}
</script>
