<script lang="ts">
  import { AlertType, AlertPosition } from "./Alert";

  const {
    message,
    type,
    position,
  }: { message: string; type: AlertType; position: AlertPosition } = $props();

  let svgData = $state("");
  let colorClass = $state("");
  let positionClass = $state("");

  switch (type) {
    case AlertType.INFO:
      colorClass = "alert-info";
      svgData = "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
      break;
    case AlertType.SUCCESS:
      colorClass = "alert-success";
      svgData = "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z";
      break;
    case AlertType.WARNING:
      colorClass = "alert-warning";
      svgData =
        "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z";
      break;
    case AlertType.ERROR:
      colorClass = "alert-error";
      svgData =
        "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z";
      break;
  }

  switch (position) {
    case AlertPosition.TOP:
      positionClass = "toast-top toast-center";
      break;
    case AlertPosition.BOTTOM_RIGHT:
      positionClass = "toast-end";
      break;
  }
</script>

<button onclick={(e) => e.currentTarget.remove()} class="alert {colorClass}">
  <svg
    xmlns="http://www.w3.org/2000/svg"
    class="h-6 w-6 shrink-0 stroke-current"
    fill="none"
    viewBox="0 0 24 24"
  >
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d={svgData}
    />
  </svg>
  <span>{message}</span>
</button>

<style>
  button {
    cursor: pointer;
  }
</style>
