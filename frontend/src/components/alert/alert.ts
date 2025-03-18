import { mount, unmount } from "svelte";
// @ts-ignore
import Alert from "./Alert.svelte";

export const enum AlertType {
  SUCCESS = 0,
  INFO = 1,
  WARNING = 2,
  ERROR = 3,
}

export const enum AlertPosition {
  TOP = 0,
  BOTTOM_RIGHT = 1,
}

const toastTop = document.getElementById("toast-top")!;
const toastBottomRight = document.getElementById("toast-bottom-right")!;

export function showAlert(
  message: string,
  type: AlertType,
  position: AlertPosition
) {
  const alert = mount(Alert, {
    target: position === AlertPosition.TOP ? toastTop : toastBottomRight,
    props: {
      message,
      type,
      position,
    },
  });

  setTimeout(() => {
    unmount(alert);
  }, 3000);
}
