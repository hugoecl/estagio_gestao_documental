<script lang="ts">
    // Accept the options data via bind:optionsJson
    // It expects the JSON string array format or null from the parent
    let { optionsJson = $bindable(null) }: { optionsJson?: string[] | null } =
        $props();

    // Internal UI state: an array of strings
    let uiOptions = $state<string[]>([]);
    let errors = $state<string | null>(null);

    // Sync internal UI state when the bound prop changes (e.g., loading existing data)
    $effect(() => {
        try {
            // If optionsJson is already an array (likely from initial state), use it directly
            if (Array.isArray(optionsJson)) {
                uiOptions = [...optionsJson]; // Create a copy
            }
            // If it's a string (likely from DB load), parse it
            else if (
                typeof optionsJson === "string" &&
                optionsJson.trim().startsWith("[")
            ) {
                const parsed = JSON.parse(optionsJson);
                if (
                    Array.isArray(parsed) &&
                    parsed.every((item) => typeof item === "string")
                ) {
                    uiOptions = parsed;
                } else {
                    throw new Error("Invalid format in optionsJson prop.");
                }
            }
            // If it's null or undefined, reset to empty
            else {
                uiOptions = [];
            }
            errors = null; // Clear errors on successful sync
        } catch (e) {
            console.error(
                "Error parsing initial options JSON:",
                optionsJson,
                e,
            );
            errors =
                "Erro ao carregar opções existentes (formato JSON inválido).";
            uiOptions = []; // Reset on error
        }
    });

    // REMOVED the $effect that syncs uiOptions -> optionsJson

    // Helper for deep array comparison
    function arraysAreEqual(
        arr1: string[] | null | undefined,
        arr2: string[] | null | undefined,
    ): boolean {
        if (arr1 === arr2) return true; // Same reference or both null/undefined
        if (!arr1 || !arr2 || arr1.length !== arr2.length) return false; // Different lengths or one is null/undefined
        for (let i = 0; i < arr1.length; i++) {
            if (arr1[i] !== arr2[i]) return false;
        }
        return true;
    }

    // Sync internal UI state ONLY when the bound prop actually changes value
    $effect(() => {
        try {
            let newUiOptions: string[] = [];
            // If optionsJson is already an array (likely from initial state), use it directly
            if (Array.isArray(optionsJson)) {
                newUiOptions = [...optionsJson]; // Create a copy
            }
            // // If it's a string (less likely now with direct binding), parse it
            // else if (typeof optionsJson === 'string' && optionsJson.trim().startsWith('[')) {
            //     const parsed = JSON.parse(optionsJson);
            //     if (Array.isArray(parsed) && parsed.every(item => typeof item === 'string')) {
            //          newUiOptions = parsed;
            //     } else {
            //         throw new Error("Invalid format in optionsJson prop string.");
            //     }
            // }
            // If it's null or undefined, reset to empty
            else {
                newUiOptions = [];
            }

            // --- Crucial Check: Only update if arrays are different ---
            if (!arraysAreEqual(uiOptions, newUiOptions)) {
                // console.log("optionsJson changed, updating uiOptions"); // Debug
                uiOptions = newUiOptions;
                errors = null; // Clear errors on successful sync
            }
        } catch (e) {
            console.error("Error parsing optionsJson prop:", optionsJson, e);
            errors = "Erro ao carregar opções (formato inválido).";
            if (!arraysAreEqual(uiOptions, [])) {
                // Avoid infinite loop if error happens repeatedly
                uiOptions = []; // Reset on error
            }
        }
    });

    // --- Update optionsJson directly when UI changes ---
    function updateParentOptions() {
        optionsJson = uiOptions.length > 0 ? [...uiOptions] : null;
        errors = null; // Clear errors on successful update
    }

    function addOption() {
        uiOptions.push("");
        uiOptions = [...uiOptions]; // Ensure reactivity
        updateParentOptions(); // Update parent state
    }

    function removeOption(index: number) {
        uiOptions.splice(index, 1);
        uiOptions = [...uiOptions]; // Ensure reactivity
        updateParentOptions(); // Update parent state
    }

    // Update function simplified as direct binding handles state change
    // Just need to update the parent state after the input event
    function handleInput(index: number, event: Event) {
        // Svelte's bind:value handles updating uiOptions[index]
        updateParentOptions(); // Update parent state after input changes
    }
</script>

<div
    class="form-control w-full md:col-span-3 space-y-2 p-3 bg-base-300 rounded-md"
>
    <div class="label pt-0">
        <span class="label-text font-medium">Opções para Seleção*</span>
    </div>
    {#if errors}
        <div class="text-error text-xs">{errors}</div>
    {/if}
    {#if uiOptions.length === 0}
        <p class="text-xs text-base-content/70 italic text-center py-2">
            Nenhuma opção adicionada.
        </p>
    {/if}
    <div class="max-h-40 overflow-y-auto space-y-2 pr-2">
        {#each uiOptions as option, index (index)}
            <div class="flex items-center gap-2">
                <input
                    type="text"
                    placeholder={`Opção ${index + 1}`}
                    class="input input-sm input-bordered flex-grow"
                    bind:value={uiOptions[index]}
                    oninput={(e) => handleInput(index, e)}
                    required
                />
                <button
                    type="button"
                    class="btn btn-xs btn-error btn-square"
                    title="Remover Opção"
                    onclick={() => removeOption(index)}
                >
                    ✕
                </button>
            </div>
        {/each}
    </div>
    <button
        type="button"
        class="btn btn-xs btn-outline btn-accent w-full mt-2"
        onclick={addOption}
    >
        <i class="fa-solid fa-plus mr-1"></i> Adicionar Opção
    </button>
</div>
