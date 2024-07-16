<template>
  <v-card class="medium-reconnect-menu" v-if="status_message">
    <v-card-title>Could not establish connection</v-card-title>
    <v-alert
      :type="status_message.type"
      class="medium-reconnect-menu__status-message"
    >
      {{ status_message.message }}
    </v-alert>
    <v-divider />
    <v-card-actions class="medium-reconnect-menu__card-actions">
      <v-btn
        class="medium-reconnect-menu__reconnect"
        size="x-large"
        variant="tonal"
        :disabled="
          reconnect_running || status_message.reconnect === 'impossible'
        "
        @click="try_reconnect()"
      >
        <v-icon
          class="medium-reconnect-menu__twitch-login-icon"
          :icon="mdiDoor"
        />
        Try to open</v-btn
      >
      <v-sheet
        rounded="rounded"
        class="medium-reconnect-menu__re-check-container"
        ><span class="medium-reconnect-menu__re-check-text">{{
          re_check_text
        }}</span
        ><v-progress-circular
          class="medium-reconnect-menu__re-check-progress"
          :size="15"
          :width="2"
          :model-value="current_check_progress"
        ></v-progress-circular
      ></v-sheet>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts" setup>
import { mdiDoor } from "@mdi/js";
import { use_medium_api } from "@/editor/api";
import { computed, onMounted, onUnmounted, Ref, ref } from "vue";
import { DataContext, use_layout_functions } from "@/editor/ui";
import { DoorStatusCheck } from "@/shared/index";
import { match } from "ts-pattern";

const { context } = defineProps<ReconnectMenu>();

export interface ReconnectMenu {
  name: "ReconnectMenu";
  context: DataContext;
}

type StatusMessage = {
  type: "warning" | "error" | "info";
  message: string;
  reconnect: "possible" | "impossible";
};

const main_door_status = ref<DoorStatusCheck["type"] | undefined>(undefined);

const current_check_progress = ref(0);
const re_check_text = ref("Rechecking...");

const { compute_input_value } = use_layout_functions();

const connection_error: Ref<
  | {
      type: string;
      message: string;
      mainDoorStatusUrl: string;
    }
  | undefined
> = computed(() => {
  const extracted_error = compute_input_value(
    {
      store: "menu_context",
      key: "connection_error",
    },
    context,
  );

  return extracted_error ? extracted_error : undefined;
});

let get_status_interval_handler: number | undefined;
let update_status_check_interval_handler: number | undefined;
onMounted(() => {
  setTimeout(() => {
    if (connection_error.value) {
      getMainDoorStatus(connection_error.value.mainDoorStatusUrl).then(
        (status) => {
          main_door_status.value = status.type as DoorStatusCheck["type"];
        },
      );
    }
  }, 100);
  main_status_check(5000);
});

const main_status_check = (time: number) => {
  if (get_status_interval_handler) {
    clearInterval(get_status_interval_handler);
  }
  if (update_status_check_interval_handler) {
    clearInterval(update_status_check_interval_handler);
  }

  re_check_text.value = "Checking door status...";
  update_status_check_interval_handler = setInterval(() => {
    if (current_check_progress.value >= 100) {
      clearInterval(update_status_check_interval_handler);
    } else {
      current_check_progress.value += time / 2500;
    }
  }, 100);

  get_status_interval_handler = setTimeout(() => {
    if (connection_error.value) {
      getMainDoorStatus(connection_error.value.mainDoorStatusUrl).then(
        (status) => {
          if (main_door_status.value === status.type) {
            re_check_text.value = "Still the same...";
          } else {
            re_check_text.value = "Uhhh something happened :o!";
          }
          main_door_status.value = status.type as DoorStatusCheck["type"];
          setTimeout(() => {
            current_check_progress.value = 0;
            setTimeout(() => {
              main_status_check(time);
            }, 2000);
          }, 2000);
        },
      );
    }
  }, time);
};

onUnmounted(() => {
  if (get_status_interval_handler) {
    clearInterval(get_status_interval_handler);
  }
  if (update_status_check_interval_handler) {
    clearInterval(update_status_check_interval_handler);
  }
});

async function getMainDoorStatus(mainDoorStatusUrl: string) {
  try {
    return (await (
      await fetch(mainDoorStatusUrl)
    ).json()) as unknown as DoorStatusCheck;
  } catch (e) {
    return { type: "unknownError", error: e };
  }
}

const status_message = computed<StatusMessage>(() => {
  if (connection_error.value) {
    if (!main_door_status.value) {
      return {
        type: "info",
        message: "Knocking on the door...",
        reconnect: "impossible",
      } as StatusMessage;
    }
    return match(main_door_status.value)
      .with(
        "open",
        (): StatusMessage => ({
          type: "info",
          message:
            "The main door is open and you should be able to come in... \n(Please tell shiku if it doesn't work!)",
          reconnect: "possible",
        }),
      )
      .with(
        "lightsOn",
        (): StatusMessage => ({
          type: "info",
          message: "You cannot come in now, I am still preparing. 😸",
          reconnect: "impossible",
        }),
      )
      .with(
        "lightsOut",
        (): StatusMessage => ({
          type: "warning",
          message:
            "Seems like nobody is home, ask shiku when the next stream will be!",
          reconnect: "impossible",
        }),
      )
      .with(
        "unknownError",
        (): StatusMessage => ({
          type: "error",
          message:
            "Something unknown went horribly wrong >_<. Ask Shiku if he can fix it!",
          reconnect: "impossible",
        }),
      )
      .with(
        "urlNotConfigured",
        (): StatusMessage => ({
          type: "error",
          message:
            "Something went horribly wrong with the url configuration >_<. Ask Shiku if he can fix it!",
          reconnect: "impossible",
        }),
      )
      .exhaustive();
  }

  return {
    type: "error",
    message:
      "Something went horribly wrong when trying to fetch the connection error >_<. Ask Shiku if he can fix it!",
    reconnect: "impossible",
  };
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

  &__status-message {
    white-space: pre;
  }

  &__card-actions {
    display: flex;
    justify-content: space-between;
  }

  &__card-actions {
    display: flex;
    justify-content: space-between;
  }

  &__re-check-container {
    display: flex;
    align-items: center;
  }

  &__re-check-progress {
    margin-left: 8px;
  }

  &__re-check-text {
    font-size: 12px;
    padding-top: 4px;
  }

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
