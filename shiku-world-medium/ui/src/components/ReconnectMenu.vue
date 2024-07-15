<template>
  <v-card class="medium-reconnect-menu">
    <v-card-title>Could not establish connection</v-card-title>
    <v-alert v-if="connection_error" type="error">
      {{ connection_error.message }}
    </v-alert>
    <v-divider />
    <v-card-actions class="medium-reconnect-menu__card-actions">
      <v-btn
        class="medium-reconnect-menu__reconnect"
        size="x-large"
        variant="tonal"
        :disabled="reconnect_running"
        @click="try_reconnect()"
      >
        <v-icon
          class="medium-reconnect-menu__twitch-login-icon"
          :icon="mdiPowerPlug"
        />
        Try reconnecting</v-btn
      >
    </v-card-actions>
  </v-card>
</template>

<script lang="ts" setup>
import { mdiPowerPlug } from "@mdi/js";
import { use_medium_api } from "@/editor/api";
import { computed, Ref, ref } from "vue";
import { DataContext, use_layout_functions } from "@/editor/ui";

const { context } = defineProps<ReconnectMenu>();

const mainDoorStatus = ref("closed");

export interface ReconnectMenu {
  name: "ReconnectMenu";
  context: DataContext;
}

const { compute_input_value } = use_layout_functions();

const connection_error: Ref<{ type: string; message: string } | undefined> =
  computed(() => {
    const extracted_error = compute_input_value(
      {
        store: "menu_context",
        key: "connection_error",
      },
      context,
    );

    return extracted_error ? extracted_error : undefined;
  });

const { reconnect } = use_medium_api();

let reconnect_running = ref(false);

function try_reconnect() {
  if (reconnect_running.value) {
    return;
  }

  reconnect_running.value = true;
  reconnect().finally(() => {
    reconnect_running.value = false;
  });
}
</script>

<style lang="scss">
.medium-reconnect-menu {
  $p: &;

  margin: 16px;
  pointer-events: all;

  &__reconnect {
    text-transform: none !important;
    color: rgb(70, 144, 255) !important;
    font-family: "Convergence", sans-serif;
    letter-spacing: 0 !important;
  }

  &__twitch-login-icon {
    --v-icon-size-multiplier: 1.25 !important;
    margin-right: 2px;
  }

  &__do-not-want {
    text-transform: none !important;
    font-family: "Convergence", sans-serif;
    letter-spacing: 0 !important;
  }

  &--flipped {
    #{$p}__do-not-want {
      transform: scale(-1, -1);
    }
  }

  &--hidden {
    #{$p}__card-actions {
      justify-content: space-between;
    }
    #{$p}__do-not-want .v-btn__content {
      color: transparent !important;
    }
  }
}
</style>
