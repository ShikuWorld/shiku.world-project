<template>
  <v-card class="medium-login-menu" :class="do_not_want_class()">
    <v-card-title>Login</v-card-title>
    <v-alert v-if="get_login_error()" type="error">
      {{ get_login_error() }}
    </v-alert>
    <v-card-text>You can only login with twitch at the moment.</v-card-text>
    <v-expand-transition v-for="click in clicks">
      <div v-if="do_not_want_count > click.count">
        <v-card-text>{{ click.message }}</v-card-text>
      </div>
    </v-expand-transition>
    <v-divider />
    <v-card-actions class="medium-login-menu__card-actions">
      <v-btn
        class="medium-login-menu__twitch-login"
        size="x-large"
        variant="tonal"
        :disabled="login_running"
        @click="login()"
      >
        <v-icon
          class="medium-login-menu__twitch-login-icon"
          :icon="mdiTwitch"
        />
        Login via Twitch </v-btn
      ><v-btn
        v-if="do_not_want_count < 61"
        class="medium-login-menu__do-not-want"
        size="x-large"
        @click="do_not_want()"
      >
        I don't wanna
      </v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts" setup>
import { mdiTwitch } from "@mdi/js";
import { use_medium_api } from "@/api";
import { ref } from "vue";
import { DataContext, use_layout_functions } from "@/ui";

const { context } = defineProps<LoginMenu>();

export interface LoginMenu {
  name: "LoginMenu";
  context: DataContext;
}

const { compute_input_value } = use_layout_functions();

const get_login_error = (): string => {
  const extracted_error = compute_input_value(
    {
      store: "current_module",
      key: "data.login_error",
    },
    context,
  );

  return extracted_error ? extracted_error : "";
};

const clicks = [
  {
    count: 0,
    message: "I only save your public twitch id and twitch username.",
  },
  {
    count: 1,
    message: "But if you don't want to that's fine. :)",
  },
  {
    count: 5,
    message: "Okay, I got it.",
  },
  {
    count: 8,
    message: "...",
  },
  {
    count: 15,
    message: "(┛ಠ_ಠ)┛彡┻━┻",
  },
  {
    count: 20,
    message: "...",
  },
  {
    count: 30,
    message: "┬─┬ ノ(ಠ_ಠ ノ)",
  },
  {
    count: 35,
    message: "Could you please stop?",
  },
  {
    count: 40,
    message: "Okay I'm going to take away the Button.",
  },
  {
    count: 43,
    message: "You found it? ;_;",
  },
  {
    count: 47,
    message: "So uh...",
  },
  {
    count: 50,
    message: "Maybe you want to log in now? :V",
  },
  {
    count: 60,
    message: "🡆a#$33rt3SUvwSri4#-🡄",
  },
];

const do_not_want_count = ref(0);

const do_not_want_class = (): string => {
  if (do_not_want_count.value > 15 && do_not_want_count.value <= 30) {
    return "medium-login-menu--flipped";
  }

  if (do_not_want_count.value > 40) {
    return "medium-login-menu--hidden";
  }

  return "";
};

const { twitch_login, communication_state } = use_medium_api();

let login_running = ref(false);

function login() {
  if (login_running.value) {
    return;
  }

  login_running.value = true;
  twitch_login(communication_state).finally(() => {
    login_running.value = false;
  });
}

function do_not_want() {
  do_not_want_count.value += 1;
}
</script>

<style lang="scss">
.medium-login-menu {
  $p: &;

  margin: 16px;
  pointer-events: all;

  &__twitch-login {
    text-transform: none !important;
    color: rgb(145, 70, 255) !important;
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
