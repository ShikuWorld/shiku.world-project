import { defineStore } from "pinia";

/* eslint-disable @typescript-eslint/no-explicit-any */
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
/* eslint-enable @typescript-eslint/no-explicit-any */
