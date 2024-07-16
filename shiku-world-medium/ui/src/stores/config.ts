import { defineStore } from "pinia";
import { AdminToSystemEvent } from "@/client/communication/api/bindings/AdminToSystemEvent";

export interface ConfigStore {
  resource_base_url: string;
  main_door_status: boolean;
  back_door_status: boolean;
}

export const use_config_store = defineStore("config", {
  state: (): ConfigStore => ({
    resource_base_url: "https://resources.shiku.world/static/",
    main_door_status: false,
    back_door_status: false,
  }),
  actions: {
    set_resource_base_url(resource_base_url: string) {
      this.resource_base_url = resource_base_url;
    },
    set_main_door_status(status: boolean) {
      this.main_door_status = status;
    },
    set_back_door_status(status: boolean) {
      this.back_door_status = status;
    },
    set_main_door_status_server(status: boolean) {
      send_admin_event({
        SetMainDoorStatus: status,
      });
    },
    set_back_door_status_server(status: boolean) {
      send_admin_event({
        SetBackDoorStatus: status,
      });
    },
  },
});

function send_admin_event(event: AdminToSystemEvent) {
  if (window.medium.communication_state.is_connection_open) {
    window.medium.communication_state.ws_connection.send(JSON.stringify(event));
  }
}
