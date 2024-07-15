import { ComponentConfig } from "@/editor/ui";

export const reconnectMenuConfig: ComponentConfig = {
  name: "HLayout",
  config: {
    columns: [
      {
        cols: { xs: 12, sm: 10, md: 8, lg: 6, xl: 5 },
        component: { name: "ReconnectMenu" },
      },
    ],
  },
};
