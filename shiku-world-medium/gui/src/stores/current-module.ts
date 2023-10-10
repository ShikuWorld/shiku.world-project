import { defineStore } from "pinia";

export interface CurrentModuleStore {
  data: any;
}

export const use_current_module_store = defineStore("currentModule", {
  state: (): CurrentModuleStore => ({
    data: {},
  }),
  actions: {
    set_data(data: any) {
      this.data = data;
    },
  },
});
