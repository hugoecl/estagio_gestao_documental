<script lang="ts">
  import calendarIcon from "@assets/calendar_icon.svg?raw";
  import "cally";
  import { onMount } from "svelte";

  let cally: HTMLButtonElement;
  let dateSpan: HTMLSpanElement;

  onMount(() => {
    cally = document.getElementById("cally") as HTMLButtonElement;
    dateSpan = cally.querySelector("span")!;
  });
</script>

<button
  popovertarget="cally-popover"
  class="input input-border"
  id="cally"
  style="anchor-name:--cally"
>
  {@html calendarIcon}

  <span>dd/mm/aaaa</span>
</button>

<div
  popover="auto"
  id="cally-popover"
  class="dropdown bg-base-100 rounded-box shadow-lg"
  style="position-anchor:--cally"
>
  <calendar-date
    class="cally"
    onclick={(e) => {
      if (e.currentTarget.value.length !== 0) {
        dateSpan.innerHTML = new Date(e.currentTarget.value).toLocaleDateString(
          "pt-PT"
        );
        cally.click();
      }
    }}
  >
    <svg
      aria-label="Previous"
      fill="none"
      class="size-4"
      slot="previous"
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      xmlns="http://www.w3.org/2000/svg"
      stroke="currentColor"
      viewBox="0 0 24 24"><path d="M15.75 19.5 8.25 12l7.5-7.5"></path></svg
    >
    <svg
      aria-label="Next"
      class="size-4"
      fill="none"
      stroke="currentColor"
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      slot="next"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"><path d="m8.25 4.5 7.5 7.5-7.5 7.5"></path></svg
    >
    <calendar-month></calendar-month>
  </calendar-date>
</div>

<style>
  calendar-month::part(heading) {
    text-transform: capitalize;
  }
</style>
