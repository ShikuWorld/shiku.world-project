import { createApp } from "vue";
import "vue3-perfect-scrollbar/dist/vue3-perfect-scrollbar.css";
import "./assets/fonts/stylesheet.css";
import "./main.scss";
import vuetify from "./plugins/vuetify";
import pinia from "./plugins/pinia";
import App from "./App.vue";
import { setup_medium_gui_api } from "./stores/store-api";
import PerfectScrollbar from "vue3-perfect-scrollbar";

createApp(App).use(PerfectScrollbar).use(vuetify).use(pinia).mount("#ui");

setup_medium_gui_api();
