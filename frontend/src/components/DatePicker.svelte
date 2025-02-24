<script lang="ts">
  import "cally";

  let date = $state("dd/mm/aaaa");
</script>

<button
  popovertarget="cally-popover1"
  class="input input-border"
  id="cally1"
  style="anchor-name:--cally1"
>
  {date}
</button>

<div
  popover
  id="cally-popover1"
  class="dropdown bg-base-100 rounded-box shadow-lg"
  style="position-anchor:--cally1"
>
  <calendar-date
    class="cally"
    onclick={(e) => {
      date = e.currentTarget.value;
      const popover = document.getElementById("cally1");
      popover.active = false;
      document.body.focus();

      // popover?.classList.add("hidden");
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
    <calendar-month onblur={(e) => console.log("toma")}></calendar-month>
  </calendar-date>
</div>
