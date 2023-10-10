import { defineStore } from "pinia";
import { ToastAlertLevel } from "@/client/communication/api/bindings/ToastAlertLevel";

export interface Toast {
  id: number;
  message: string;
  level: ToastAlertLevel;
  start_time: number;
  duration: number;
  progress: number;
}

export interface ToastStore {
  toast_count: number;
  toast_map: { [id: number]: Toast };
}

export const use_toast_store = defineStore("toast", {
  state: (): ToastStore => ({
    toast_map: {},
    toast_count: 0,
  }),
  actions: {
    add_toast(message: string, level: ToastAlertLevel, duration?: number) {
      this.toast_count = this.toast_count + 1;

      this.toast_map = {
        ...this.toast_map,
        [this.toast_count]: {
          id: this.toast_count,
          message,
          level,
          duration: duration ? duration : 30000,
          start_time: Date.now(),
          progress: 0,
        },
      };
    },
    remove_toast(id: number) {
      delete this.toast_map[id];
    },
    get_toasts_by_asc_date(): Toast[] {
      return Object.values(this.toast_map).sort((a, b) => {
        return a.start_time - b.start_time;
      });
    },
    update_toast_progress() {
      for (const toast of Object.values(this.toast_map)) {
        const now = Date.now() - toast.start_time;
        const end = toast.duration;
        toast.progress = (now * 100) / end;
        if (toast.progress > 100) {
          this.remove_toast(toast.id);
        }
      }
    },
  },
});
