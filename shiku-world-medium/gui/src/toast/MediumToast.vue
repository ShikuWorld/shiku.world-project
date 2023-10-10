<template>
  <div
    class="medium-toast__alert-wrapper"
    :class="{ 'medium-toast--wide': display.mdAndUp }"
  >
    <v-alert
      class="medium-toast__toast"
      v-for="toast in toast_store.get_toasts_by_asc_date()"
      :key="toast.id"
      density="comfortable"
      :type="map_alert_level(toast.level)"
      variant="tonal"
    >
      <span class="medium-toast__message">{{ toast.message }}</span>
      <span
        class="medium-toast__delete"
        @click="toast_store.remove_toast(toast.id)"
        ><v-icon size="x-small" :icon="mdiCloseThick"></v-icon
      ></span>
      <v-progress-linear
        class="medium-toast__progress"
        height="1"
        :model-value="toast.progress"
      ></v-progress-linear>
    </v-alert>
  </div>
</template>

<style lang="scss">
.medium-toast {
  pointer-events: all;
  width: 50%;
  padding: 8px 8px 0 0;

  &--wide {
    width: 33.3%;
  }

  &__progress {
    position: absolute !important;
    bottom: 2px;
    left: 0;
  }

  &__toast {
    position: relative;
    background-color: rgba(0, 0, 0, 0.8) !important;
  }

  &__alert-wrapper {
    display: flex;
    flex-direction: column;
  }

  &__delete {
    position: absolute;
    top: 0;
    right: 0;
    cursor: pointer;
    color: rgb(var(--v-theme-error)) !important;
  }
}
</style>

<script lang="ts" setup>
import { use_toast_store } from "@/stores/toast";
import { ToastAlertLevel } from "../../../client/communication/api/bindings/ToastAlertLevel";
import { match } from "ts-pattern";
import { mdiCloseThick } from "@mdi/js";
import { useDisplay } from "vuetify";
import { ref } from "vue";

const toast_store = use_toast_store();
const display = ref(useDisplay());

const map_alert_level = (level: ToastAlertLevel): string =>
  match(level)
    .with("Success", () => "success")
    .with("Error", () => "error")
    .with("Info", () => "info")
    .exhaustive();

setInterval(() => toast_store.update_toast_progress(), 1000);
</script>
