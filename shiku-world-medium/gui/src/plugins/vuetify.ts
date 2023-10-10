// Styles
import "@mdi/font/css/materialdesignicons.css";

import { aliases, mdi } from "vuetify/iconsets/mdi-svg";
import { createVuetify } from "vuetify";

export default createVuetify({
  theme: {
    defaultTheme: "dark",
    themes: {
      dark: {
        dark: true,
        colors: {
          primary: "#37474f",
          secondary: "#cfd8dc",
        },
      },
    },
  },
  icons: {
    defaultSet: "mdi",
    aliases,
    sets: {
      mdi,
    },
  },
});
