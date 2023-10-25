import { defineStore } from "pinia";

export interface ConfigStore {
  resource_base_url: string;
}

export const use_config_store = defineStore("config", {
  state: (): ConfigStore => ({
    resource_base_url: "",
  }),
  actions: {
    set_resource_base_url(resource_base_url: string) {
      this.resource_base_url = resource_base_url;
    },
  },
});
