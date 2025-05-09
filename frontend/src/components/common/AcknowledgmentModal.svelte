<script lang="ts">
    let {
        isOpen = $bindable(),
        recordDisplayName,
        onConfirm,
        onCancel,
    }: {
        isOpen: boolean;
        recordDisplayName: string;
        onConfirm: () => void;
        onCancel: () => void;
    } = $props();

    let modalRef: HTMLDialogElement;

    $effect(() => {
        if (modalRef) {
            if (isOpen && !modalRef.open) {
                modalRef.showModal();
            } else if (!isOpen && modalRef.open) {
                modalRef.close();
            }
        }
    });

    function handleConfirm() {
        onConfirm();
    }

    function handleCancel() {
        isOpen = false;
        onCancel();
    }

    function handleDialogClose() {
        if (isOpen) {
            isOpen = false;
            onCancel();
        }
    }
</script>

<dialog class="modal" bind:this={modalRef} onclose={handleDialogClose}>
    <div class="modal-box">
        <form method="dialog">
            <button
                class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                onclick={handleCancel}>✕</button
            >
        </form>
        <h3 class="font-bold text-lg">Confirmação de Leitura</h3>
        <p class="py-4">
            Ao prosseguir, confirmo que tomei conhecimento do conteúdo do
            registo: <strong class="text-primary">{recordDisplayName}</strong>.
        </p>
        <p class="text-xs text-base-content/70">Esta ação será registada.</p>
        <div class="modal-action mt-6">
            <button class="btn btn-ghost" onclick={handleCancel}
                >Cancelar</button
            >
            <button class="btn btn-primary" onclick={handleConfirm}
                >Confirmar e Abrir</button
            >
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button onclick={handleCancel}>close</button>
    </form>
</dialog>
