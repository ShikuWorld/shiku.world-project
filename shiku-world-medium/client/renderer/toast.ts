import { ToastAlertLevel } from "../communication/api/bindings/ToastAlertLevel";

export function createToast(alertLevel: ToastAlertLevel, text: string) {
  window.medium_gui.toast.add_toast(text, alertLevel);
}
