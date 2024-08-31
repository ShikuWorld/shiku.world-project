<template>
  <v-card class="medium-reconnect-menu" v-if="status_message">
    <v-card-title class="medium-reconnect-menu__title"
      >Could not establish connection
    </v-card-title>
    <div class="medium-reconnect-menu__discord-icon-wrap">
      <v-tooltip text="Join the Discord!">
        <template v-slot:activator="{ props }">
          <a :href="discord_link" target="_blank"
            ><v-icon
              class="medium-reconnect-menu__discord-icon"
              :icon="discord_icon"
              size="20"
              v-bind="props"
            ></v-icon
          ></a>
        </template>
      </v-tooltip>
    </div>
    <v-alert
      :type="status_message.type"
      class="medium-reconnect-menu__status-message"
      v-html="status_message.message"
    ></v-alert>
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
        ><span
          class="medium-reconnect-menu__re-check-text"
          v-html="re_check_text"
        ></span
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

const discord_link = "https://discord.gg/qsbb3Trx";
const discord_icon =
  "M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z";

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

let update_status_check_interval_handler: number | undefined;
let status_check_running = false;
let unmounted = false;
onMounted(() => {
  window.setTimeout(() => {
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
  if (status_check_running || unmounted) {
    return;
  }
  status_check_running = true;

  if (update_status_check_interval_handler) {
    clearInterval(update_status_check_interval_handler);
  }
  re_check_text.value = "Checking door status...";
  update_status_check_interval_handler = window.setInterval(() => {
    if (current_check_progress.value >= 100) {
      clearInterval(update_status_check_interval_handler);
    } else {
      current_check_progress.value += time / 2500;
    }
  }, 100);

  window.setTimeout(() => {
    if (connection_error.value) {
      getMainDoorStatus(connection_error.value.mainDoorStatusUrl).then(
        (status) => {
          if (main_door_status.value === status.type) {
            re_check_text.value =
              main_door_status.value === "open"
                ? "Still open. :)"
                : "Still closed...";
          } else {
            re_check_text.value = "Uhhh something happened :o!";
          }
          main_door_status.value = status.type as DoorStatusCheck["type"];
          window.setTimeout(() => {
            current_check_progress.value = 0;
            window.setTimeout(() => {
              status_check_running = false;
              main_status_check(time);
            }, 2000);
          }, 2000);
        },
      );
    }
  }, time);
};

onUnmounted(() => {
  unmounted = true;
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
          message: `Seems like nobody is home, ask <a href="${discord_link}">Shiku</a> when the next stream will be!`,
          reconnect: "impossible",
        }),
      )
      .with(
        "unknownError",
        (): StatusMessage => ({
          type: "error",
          message: `Something unknown went horribly wrong >_<. Ask <a href="${discord_link}">Shiku</a> if he can fix it!`,
          reconnect: "impossible",
        }),
      )
      .with(
        "urlNotConfigured",
        (): StatusMessage => ({
          type: "error",
          message: `Something went horribly wrong with the url configuration >_<. Ask <a href="${discord_link}">Shiku</a> if he can fix it!`,
          reconnect: "impossible",
        }),
      )
      .exhaustive();
  }

  return {
    type: "error",
    message: `Something went horribly wrong when trying to fetch the connection error >_<. Ask <a href="${discord_link}">Shiku</a> if he can fix it!`,
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
  position: relative;
  $p: &;

  margin: 16px;
  pointer-events: all;

  &__title {
    display: flex;
    justify-content: space-between;
  }

  &__discord-icon-wrap {
    position: absolute;
    top: 12px;
    right: 16px;
  }

  &__discord-icon {
    color: #57f287;
    cursor: pointer;
  }

  &__status-message {
    white-space: pre-wrap;

    a {
      color: #57f287;
      display: contents;
    }
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
