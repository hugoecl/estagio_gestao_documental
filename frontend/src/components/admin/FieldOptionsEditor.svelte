<script lang="ts">
    let { optionsJson = $bindable(null) }: { optionsJson?: string[] | null } =
        $props();

    let uiOptions = $state<string[]>([]);
    let errors = $state<string | null>(null);
    let internalUpdate = false; // Flag to prevent effect recursion

    // Effect 1: Sync from parent (optionsJson) to local (uiOptions)
    $effect(() => {
        // Only update from prop if not an internal update and actual change
        if (internalUpdate) {
            internalUpdate = false; // Reset flag
            return;
        }
        try {
            let newUiOptions: string[] = [];
            if (Array.isArray(optionsJson)) {
                newUiOptions = [...optionsJson];
            } else {
                newUiOptions = [];
            }

            if (!arraysAreEqual(uiOptions, newUiOptions)) {
                console.log(
                    "FieldOptionsEditor: Syncing optionsJson to uiOptions",
                    newUiOptions,
                );
                uiOptions = newUiOptions;
                errors = null;
            }
        } catch (e) {
            console.error(
                "FieldOptionsEditor: Error parsing optionsJson prop:",
                optionsJson,
                e,
            );
            errors = "Erro ao carregar opções (formato inválido).";
            if (!arraysAreEqual(uiOptions, [])) {
                uiOptions = [];
            }
        }
    });

    // Effect 2: Sync from local (uiOptions) to parent (optionsJson)
    $effect(() => {
        console.log(
            "FieldOptionsEditor: uiOptions changed, preparing to update optionsJson",
            uiOptions,
        );
        internalUpdate = true; // Signal that the next optionsJson change is from here
        optionsJson = uiOptions.length > 0 ? [...uiOptions] : null;
        // No need to clear errors here unless this effect itself causes an error
    });

    function arraysAreEqual(
        arr1: string[] | null | undefined,
        arr2: string[] | null | undefined,
    ): boolean {
        if (arr1 === arr2) return true;
        if (!arr1 || !arr2 || arr1.length !== arr2.length) return false;
        for (let i = 0; i < arr1.length; i++) {
            if (arr1[i] !== arr2[i]) return false;
        }
        return true;
    }

    function addOption() {
        // This will mutate uiOptions, triggering Effect 2
        uiOptions.push("");
        uiOptions = [...uiOptions]; // Ensure reactivity for Svelte 5 if .push isn't enough
        console.log("FieldOptionsEditor: addOption, uiOptions:", uiOptions);
    }

    function removeOption(index: number) {
        // This will mutate uiOptions, triggering Effect 2
        uiOptions.splice(index, 1);
        uiOptions = [...uiOptions]; // Ensure reactivity
        console.log("FieldOptionsEditor: removeOption, uiOptions:", uiOptions);
    }

    // The bind:value on the input will directly update uiOptions[index].
    // Effect 2 will then pick up this change to uiOptions and propagate it to optionsJson.
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
