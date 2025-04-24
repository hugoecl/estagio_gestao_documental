<script lang="ts">
    import { isAdmin } from "@stores/auth-store";
    import { logoutUser } from "@api/auth-api";
    import { onMount } from "svelte";

    let detailsElement: HTMLDetailsElement | null = null;
    let svgElement: SVGElement | null = null;
    let showAdminLink = $derived(isAdmin ?? false); // Use derived state

    function toggleRotate(open: boolean) {
        if (svgElement) {
            if (open) {
                svgElement.classList.add("rotate");
            } else {
                svgElement.classList.remove("rotate");
            }
        }
    }

    async function handleLogout() {
        await logoutUser();
        // Redirect after logout
        if (typeof window !== "undefined") {
            window.location.href = "/iniciar-sessao/";
        }
    }

    // Handle closing dropdown when clicking outside
    function handleClickOutside(event: MouseEvent) {
        if (detailsElement && !detailsElement.contains(event.target as Node)) {
            if (detailsElement.open) {
                detailsElement.open = false; // Close the dropdown
                toggleRotate(false);
            }
        }
    }

    onMount(() => {
        if (typeof document !== "undefined") {
            document.addEventListener("click", handleClickOutside, true);
        }
        return () => {
            if (typeof document !== "undefined") {
                document.removeEventListener("click", handleClickOutside, true);
            }
        };
    });
</script>

<details
    class="dropdown dropdown-end"
    bind:this={detailsElement}
    ontoggle={() => toggleRotate(detailsElement?.open ?? false)}
>
    <summary
        tabindex="0"
        role="button"
        class="btn btn-ghost btn-circle avatar m-1"
    >
        <!-- Use avatar style for circle -->
        <div class="w-7 rounded-full">
            <!-- Adjust size --
             <!-- Placeholder Icon - Replace with user avatar later if needed -->
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="w-full h-full"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M17.982 18.725A7.488 7.488 0 0 0 12 15.75a7.488 7.488 0 0 0-5.982 2.975m11.963 0a9 9 0 1 0-11.963 0m11.963 0A8.966 8.966 0 0 1 12 21a8.966 8.966 0 0 1-5.982-2.275M15 9.75a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                />
            </svg>
        </div>
        <svg
            bind:this={svgElement}
            fill="none"
            viewBox="0 0 24 24"
            class="w-7 h-7 cursor-pointer absolute opacity-0"
            xmlns="http://www.w3.org/2000/svg"
            id="dropdown-settings-icon-visual"
        >
            <path
                clip-rule="evenodd"
                stroke="currentColor"
                d="m14 4.3117c0-.72443-.5873-1.3117-1.3117-1.3117h-1.3766c-.7244 0-1.3117.58727-1.3117 1.3117v.25706c0 .42782-.28712.79949-.68224.96353-.39526.16411-.85548.10157-1.1581-.20106l-.27666-.27665c-.58242-.58243-1.52672-.58243-2.10915 0l-.71928.71928c-.58242.58242-.58242 1.52672 0 2.10914l.27666.27666c.30263.30263.36517.76284.20106 1.1581-.16404.39512-.53571.68224-.96352.68224h-.25707c-.72443 0-1.3117.5873-1.3117 1.3117v1.3766c0 .7244.58727 1.3117 1.3117 1.3117h.40807c.33822 0 .63289.2259.75572.5411.11803.3028.06177.6495-.16807.8794l-.31838.3184c-.54624.5462-.54624 1.4318 0 1.9781l.96484.9648c.48299.483 1.26607.483 1.74906 0l.04451-.0445c.37602-.376.95241-.4395 1.43926-.2256.46838.2058.81329.6492.81329 1.1608v.1158c0 .7244.5873 1.3117 1.3117 1.3117h1.3766c.7244 0 1.3117-.5873 1.3117-1.3117v-.2571c0-.4278.2871-.7995.6822-.9635.3953-.1641.8555-.1016 1.1581.2011l.0871.0871c.4421.4421 1.1589.4421 1.601 0l1.2275-1.2275c.4421-.4421.4421-1.1589 0-1.601l-.0871-.0871c-.3027-.3026-.3652-.7628-.2011-1.1581.164-.3951.5357-.6822.9635-.6822h.2571c.7244 0 1.3117-.5873 1.3117-1.3117v-1.3766c0-.7244-.5873-1.3117-1.3117-1.3117h-.1158c-.5116 0-.955-.34491-1.1608-.81329-.2139-.48685-.1504-1.06324.2256-1.43926l.0445-.0445c.483-.48299.483-1.26606 0-1.74904l-.9649-.96487c-.5462-.54623-1.4318-.54623-1.978 0l-.3184.31838c-.2299.22984-.5766.2861-.8794.16807-.3152-.12283-.5411-.4175-.5411-.75573zm-2 10.6883c1.6569 0 3-1.3431 3-3s-1.3431-3-3-3-3 1.3431-3 3 1.3431 3 3 3z"
                fill-rule="evenodd"
                stroke-linejoin="round"
                stroke-width="1.5"
            ></path>
        </svg>
    </summary>
    <ul
        tabindex="0"
        class="dropdown-content menu bg-base-200 rounded-box z-[1] w-52 p-2 shadow mt-2"
    >
        {#if showAdminLink}
            <li>
                <a href="/admin/"
                    ><i class="fa-solid fa-user-shield w-4 mr-2"></i>Admin</a
                >
            </li>
            <li class="menu-title"><span>Utilizador</span></li>
        {/if}
        <li>
            <a href="/settings/"
                ><i class="fa-solid fa-cog w-4 mr-2"></i>Definições</a
            >
        </li>
        <li>
            <button onclick={handleLogout}
                ><i class="fa-solid fa-sign-out-alt w-4 mr-2"></i>Terminar
                Sessão</button
            >
        </li>
    </ul>
</details>

<style>
    /* Keep the rotate style */
    #dropdown-settings-icon-visual {
        /* Target the visual icon if you keep it */
        transition: transform 0.3s;
    }
    .rotate {
        transform: rotate(90deg);
    }
    /* Ensure summary doesn't have extra padding if using avatar */
    summary.avatar {
        padding: 0;
    }
</style>
