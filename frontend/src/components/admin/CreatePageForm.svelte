<script lang="ts">
    import { onMount, tick } from "svelte";
    // ... other imports
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    import FieldOptionsEditor from "./FieldOptionsEditor.svelte"; // Import the new component
    import { getFieldTypes, getValidations } from "@api/fields-api";
    import { getRoles } from "@api/roles-api";
    import type {
        CreateCustomPageRequest,
        RolePermissionRequest,
    } from "@lib/types/custom-page";
    import type { Role } from "@lib/types/roles";
    import type {
        CreatePageFieldRequest,
        FieldType as BackendFieldType, // Rename to avoid clash if needed
        ValidationFunction,
    } from "@lib/types/fields";
    import { createCustomPage } from "@api/custom-pages-api";

    // --- State ---
    // ... (pageData, fields, permissions, etc. - no change needed here initially)
    let pageData = $state<Partial<CreateCustomPageRequest>>({
        is_group: false, // Default to page
    });
    let fields = $state<CreatePageFieldRequest[]>([]);
    let permissions = $state<Record<number, RolePermissionRequest>>({});
    let fieldTypes = $state<BackendFieldType[]>([]);
    let validations = $state<ValidationFunction[]>([]);
    let roles = $state<Role[]>([]);
    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});

    // --- Fetch Initial Data ---
    onMount(async () => {
        // ... (fetching logic remains the same)
        try {
            const [fetchedFieldTypes, fetchedValidations, fetchedRoles] =
                await Promise.all([
                    getFieldTypes(),
                    getValidations(),
                    getRoles(),
                ]);
            fieldTypes = fetchedFieldTypes;
            validations = fetchedValidations;
            roles = fetchedRoles;
            const initialPermissions: Record<number, RolePermissionRequest> =
                {};
            roles.forEach((role) => {
                initialPermissions[role.id] = {
                    role_id: role.id,
                    can_view: false,
                    can_create: false,
                    can_edit: false,
                    can_delete: false,
                    can_manage_fields: false,
                };
            });
            permissions = initialPermissions;
        } catch (e: any) {
            showAlert(
                `Erro ao carregar dados iniciais: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isLoading = false;
        }
    });

    // --- Field Management ---
    function addField() {
        // ... (addField logic remains the same, ensure options: null initially)
        fields.push({
            name: `campo_${fields.length + 1}`,
            display_name: "",
            field_type_id: fieldTypes[0]?.id || 1,
            required: false,
            options: null, // Initialize options as null
            validation_name: null,
            is_searchable: true,
            is_displayed_in_table: true,
            order_index: fields.length,
            notification_enabled: false, // Initialize notification fields
            notification_days_before: null,
            notification_target_date_part: null,
        });
        fields = [...fields];
    }

    function removeField(index: number) {
        // ... (removeField logic remains the same)
        fields.splice(index, 1);
        fields.forEach((field, i) => (field.order_index = i));
        fields = [...fields];
    }

    function handleFieldNameChange(index: number, event: Event) {
        // ... (logic remains the same)
        const input = event.target as HTMLInputElement;
        fields[index].name = input.value
            .toLowerCase()
            .replace(/\s+/g, "_")
            .replace(/[^a-z0-9_]/g, "");
        fields = [...fields];
    }

    function getFieldTypeName(fieldTypeId: number): string {
        return (
            fieldTypes.find((ft) => ft.id === fieldTypeId)?.name ?? "UNKNOWN"
        );
    }

    // Helper to get field type object
    function getFieldTypeById(fieldTypeId: number): BackendFieldType | undefined {
        return fieldTypes.find((ft) => ft.id === fieldTypeId);
    }

    // --- *** REMOVE parseOptions function - logic moved to FieldOptionsEditor *** ---
    // function parseOptions(index: number, value: string) { ... } // DELETE THIS

    // --- Form Submission ---
    async function handleSubmit(e: Event) {
        e.preventDefault();
        isSubmitting = true;
        errors = {};

        // --- Validation (Simplified - full validation omitted for brevity) ---
        if (!pageData.name) errors["page_name"] = "Nome obrigatório.";
        if (!pageData.path) errors["page_path"] = "Caminho obrigatório.";
        else if (!/^[a-z0-9\/-]+$/.test(pageData.path))
            errors["page_path"] = "Caminho inválido.";

        if (!pageData.is_group) {
            fields.forEach((field, index) => {
                if (!field.display_name)
                    errors[`field_${index}_display_name`] = "Obrigatório.";
                if (!field.name) errors[`field_${index}_name`] = "Obrigatório.";
                else if (!/^[a-z0-9_]+$/.test(field.name))
                    errors[`field_${index}_name`] = "Inválido.";
                // *** Validation for options is handled implicitly by FieldOptionsEditor ***
                // We just need to ensure the final `field.options` is correct before sending
            });
            if (fields.length === 0)
                errors["fields_general"] = "Pelo menos um campo é necessário.";
        }
        // --- End Validation ---

        if (Object.keys(errors).length > 0) {
            showAlert(
                "Existem erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            isSubmitting = false;
            errors = { ...errors };
            return;
        }

        // --- Prepare Data ---
        // Format paths (same as before)
        let formattedPath = pageData.path!.trim().toLowerCase();
        if (!formattedPath.startsWith("/")) formattedPath = "/" + formattedPath;
        if (formattedPath.length > 1 && formattedPath.endsWith("/"))
            formattedPath = formattedPath.slice(0, -1);
        let formattedParentPath =
            pageData.parent_path?.trim().toLowerCase() || null;
        if (formattedParentPath) {
            if (!formattedParentPath.startsWith("/"))
                formattedParentPath = "/" + formattedParentPath;
            if (
                formattedParentPath.length > 1 &&
                formattedParentPath.endsWith("/")
            )
                formattedParentPath = formattedParentPath.slice(0, -1);
        }

        // *** IMPORTANT: Ensure field.options contains the JSON value (string array) ***
        // The FieldOptionsEditor's binding handles converting the UI array to the correct JSON format
        // for the `field.options` property before this point.

        const finalData: CreateCustomPageRequest = {
            name: pageData.name!,
            path: formattedPath,
            parent_path: formattedParentPath,
            is_group: pageData.is_group!,
            description: pageData.description || null,
            icon: pageData.icon || null,
            fields: pageData.is_group
                ? []
                : fields.map((f) => ({
                      // Ensure null options if empty
                      ...f,
                      options: f.options ?? null,
                      // Ensure notification fields are null if not enabled
                      notification_days_before: f.notification_enabled
                          ? f.notification_days_before
                          : null,
                      notification_target_date_part: f.notification_enabled
                          ? f.notification_target_date_part
                          : null,
                  })),
            permissions: pageData.is_group ? [] : Object.values(permissions),
        };
        // --- End Prepare Data ---

        // --- API Call ---
        try {
            // ... (API call logic remains the same) ...
            const result = await createCustomPage(finalData);
            if (result.success) {
                showAlert(
                    `${pageData.is_group ? "Grupo" : "Página"} criado com sucesso!`,
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                if (typeof window !== "undefined")
                    window.location.href = "/admin/pages/";
            } else {
                throw new Error("Falha ao criar página/grupo no backend.");
            }
        } catch (e: any) {
            showAlert(
                `Erro ao criar ${pageData.is_group ? "grupo" : "página"}: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmitting = false;
        }
        // --- End API Call ---
    }
</script>

{#if isLoading}
    <div class="flex justify-center items-center p-10">
        <span class="loading loading-lg loading-spinner"></span>
    </div>
{:else}
    <form
        onsubmit={handleSubmit}
        class="space-y-6 p-4 bg-base-100 rounded-lg shadow border border-base-content/10"
    >
        <!-- Page/Group Details Fieldset -->
        <fieldset
            class="grid grid-cols-1 md:grid-cols-2 gap-4 border p-4 rounded-md border-base-content/20"
        >
            <legend class="text-lg font-semibold px-2"
                >Detalhes {pageData.is_group ? "do Grupo" : "da Página"}</legend
            >
            <label class="form-control w-full">
                <div class="label"><span class="label-text">Nome*</span></div>
                <input
                    type="text"
                    placeholder={pageData.is_group
                        ? "Ex: Gestão Interna"
                        : "Ex: Licenças de Software"}
                    class="input input-bordered w-full"
                    bind:value={pageData.name}
                    required
                />
                {#if errors.page_name}<span class="text-error text-xs mt-1"
                        >{errors.page_name}</span
                    >{/if}
            </label>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Caminho (URL/Base)*</span>
                </div>
                <input
                    type="text"
                    placeholder={pageData.is_group
                        ? "Ex: /gestao"
                        : "Ex: /gestao/licencas-software"}
                    class="input input-bordered w-full"
                    bind:value={pageData.path}
                    required
                />
                {#if errors.page_path}<span class="text-error text-xs mt-1"
                        >{errors.page_path}</span
                    >{/if}
                <div class="label">
                    <span class="label-text-alt"
                        >Será formatado (minúsculas, -, /). Sem / no final.</span
                    >
                </div>
            </label>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Caminho Pai (Opcional)</span>
                </div>
                <input
                    type="text"
                    placeholder="Ex: /gestao ou deixe em branco"
                    class="input input-bordered w-full"
                    bind:value={pageData.parent_path}
                />
                <div class="label">
                    <span class="label-text-alt"
                        >Para menus aninhados. Será formatado.</span
                    >
                </div>
            </label>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Ícone (FontAwesome, Opcional)</span
                    >
                </div>
                <input
                    type="text"
                    placeholder="Ex: user-shield (apenas nome)"
                    class="input input-bordered w-full"
                    bind:value={pageData.icon}
                />
                <div class="label">
                    <span class="label-text-alt"
                        ><a
                            href="https://fontawesome.com/search?m=free&s=solid"
                            target="_blank"
                            class="link link-primary">Ver ícones</a
                        ></span
                    >
                </div>
            </label>
            <label class="form-control w-full md:col-span-2">
                <div class="label">
                    <span class="label-text">Descrição (Opcional)</span>
                </div>
                <textarea
                    placeholder="Breve descrição da finalidade"
                    class="textarea textarea-bordered w-full"
                    bind:value={pageData.description}
                ></textarea>
            </label>

            <!-- Is Group Checkbox -->
            <div class="form-control md:col-span-2">
                <label class="label cursor-pointer justify-start gap-2">
                    <input
                        type="checkbox"
                        class="toggle toggle-primary"
                        bind:checked={pageData.is_group}
                    />
                    <span class="label-text font-medium"
                        >É um Grupo/Pasta (sem registos)?</span
                    >
                </label>
                <div class="label">
                    <span class="label-text-alt"
                        >Marque se isto for apenas uma pasta no menu para
                        organizar outras páginas.</span
                    >
                </div>
            </div>
        </fieldset>

        {#if !pageData.is_group}
            <!-- Fields Section -->
            <fieldset
                class="border p-4 rounded-md border-base-content/20 space-y-4"
            >
                <legend class="text-lg font-semibold px-2"
                    >Campos do Registo</legend
                >
                {#if errors.fields_general}<p
                        class="text-error text-sm text-center -mt-2 mb-2"
                    >
                        {errors.fields_general}
                    </p>{/if}

                {#each fields as field, index (index)}
                    <div class="border p-3 rounded bg-base-200 relative">
                        <button
                            type="button"
                            class="btn btn-xs btn-error absolute top-2 right-2"
                            title="Remover Campo"
                            onclick={() => removeField(index)}>✕</button
                        >
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                            <label class="form-control w-full">
                                <div class="label">
                                    <span class="label-text"
                                        >Nome Exibição*</span
                                    >
                                </div>
                                <input
                                    type="text"
                                    placeholder="Ex: Nome da Licença"
                                    class="input input-sm input-bordered w-full"
                                    bind:value={field.display_name}
                                    required
                                    oninput={(e) =>
                                        handleFieldNameChange(index, e)}
                                />
                                {#if errors[`field_${index}_display_name`]}<span
                                        class="text-error text-xs mt-1"
                                        >{errors[
                                            `field_${index}_display_name`
                                        ]}</span
                                    >{/if}
                            </label>
                            <label class="form-control w-full">
                                <div class="label">
                                    <span class="label-text">Nome Interno*</span
                                    >
                                </div>
                                <input
                                    type="text"
                                    placeholder="Ex: nome_licenca"
                                    class="input input-sm input-bordered w-full bg-base-300"
                                    bind:value={field.name}
                                    required
                                    readonly
                                />
                                {#if errors[`field_${index}_name`]}<span
                                        class="text-error text-xs mt-1"
                                        >{errors[`field_${index}_name`]}</span
                                    >{/if}
                                <div class="label">
                                    <span class="label-text-alt"
                                        >Gerado automaticamente (minúsculas, _)</span
                                    >
                                </div>
                            </label>
                            <label class="form-control w-full">
                                <div class="label">
                                    <span class="label-text">Tipo*</span>
                                </div>
                                <select
                                    class="select select-sm select-bordered w-full"
                                    bind:value={field.field_type_id}
                                    required
                                    onchange={() => {
                                        // Reset options and notification settings if type changes
                                        fields[index].options = null;
                                        fields[index].notification_enabled = false;
                                        fields[index].notification_days_before = null;
                                        fields[index].notification_target_date_part = null;
                                        fields = [...fields]; // Trigger reactivity
                                    }}
                                >
                                    {#each fieldTypes as ft}
                                        <option value={ft.id}>{ft.name}</option>
                                    {/each}
                                </select>
                            </label>

                            <!-- *** NEW: Conditional Options Editor *** -->
                            {#if getFieldTypeName(field.field_type_id) === "SELECT"}
                                <div class="md:col-span-3">
                                    <FieldOptionsEditor
                                        bind:optionsJson={field.options}
                                        {fieldTypes}
                                        fieldTypeId={field.field_type_id}
                                    />
                                    {#if errors[`field_${index}_options`]}<span
                                            class="text-error text-xs mt-1"
                                            >{errors[
                                                `field_${index}_options`
                                            ]}</span
                                        >{/if}
                                </div>
                            {/if}

                            {#if getFieldTypeName(field.field_type_id) === "DATE_RANGE"}
                                <div
                                    class="md:col-span-3 grid grid-cols-1 md:grid-cols-3 gap-3 border-t border-base-content/10 pt-3 mt-3"
                                >
                                    <div class="form-control md:col-span-1">
                                        <label
                                            class="label cursor-pointer justify-start gap-2"
                                        >
                                            <input
                                                type="checkbox"
                                                class="toggle toggle-primary toggle-sm"
                                                bind:checked={field.notification_enabled}
                                            />
                                            <span class="label-text"
                                                >Ativar Notificação?</span
                                            >
                                        </label>
                                    </div>

                                    {#if field.notification_enabled}
                                        <label class="form-control w-full">
                                            <div class="label pb-0">
                                                <span class="label-text"
                                                    >Notificar Dias Antes*</span
                                                >
                                            </div>
                                            <input
                                                type="number"
                                                min="1"
                                                placeholder="Ex: 7"
                                                class="input input-sm input-bordered w-full"
                                                bind:value={field.notification_days_before}
                                                required={field.notification_enabled}
                                            />
                                        </label>
                                        <label class="form-control w-full">
                                            <div class="label pb-0">
                                                <span class="label-text"
                                                    >Referente a*</span
                                                >
                                            </div>
                                            <select
                                                class="select select-sm select-bordered w-full"
                                                bind:value={field.notification_target_date_part}
                                                required={field.notification_enabled}
                                            >
                                                <option value={null} disabled
                                                    >Selecione...</option
                                                >
                                                <option value="start_date"
                                                    >Data de Início</option
                                                >
                                                <option value="end_date"
                                                    >Data de Fim</option
                                                >
                                                <!-- Add other parts if applicable -->
                                            </select>
                                        </label>
                                    {/if}
                                </div>
                            {/if}
                            <!-- *** END NEW *** -->

                            <!-- Validation -->
                            <label class="form-control w-full">
                                <div class="label">
                                    <span class="label-text"
                                        >Validação (Opcional)</span
                                    >
                                </div>
                                <select
                                    class="select select-sm select-bordered w-full"
                                    bind:value={field.validation_name}
                                >
                                    <option value={null}>Nenhuma</option>
                                    {#each validations as v}
                                        <option value={v.name}
                                            >{v.name} ({v.description})</option
                                        >
                                    {/each}
                                </select>
                            </label>

                            <!-- Checkboxes -->
                            <div class="form-control">
                                <label
                                    class="label cursor-pointer justify-start gap-2"
                                >
                                    <input
                                        type="checkbox"
                                        class="checkbox checkbox-sm"
                                        bind:checked={field.required}
                                    />
                                    <span class="label-text">Obrigatório</span>
                                </label>
                            </div>
                            <div class="form-control">
                                <label
                                    class="label cursor-pointer justify-start gap-2"
                                >
                                    <input
                                        type="checkbox"
                                        class="checkbox checkbox-sm"
                                        bind:checked={field.is_searchable}
                                    />
                                    <span class="label-text">Pesquisável</span>
                                </label>
                            </div>
                            <div class="form-control">
                                <label
                                    class="label cursor-pointer justify-start gap-2"
                                >
                                    <input
                                        type="checkbox"
                                        class="checkbox checkbox-sm"
                                        bind:checked={
                                            field.is_displayed_in_table
                                        }
                                    />
                                    <span class="label-text"
                                        >Mostrar na Tabela</span
                                    >
                                </label>
                            </div>

                            <input
                                type="hidden"
                                bind:value={field.order_index}
                            />
                        </div>
                    </div>
                {/each}

                <button
                    type="button"
                    class="btn btn-sm btn-outline btn-accent"
                    onclick={addField}
                    ><i class="fa-solid fa-plus mr-1"></i> Adicionar Campo</button
                >
                {#if fields.length === 0}<p
                        class="text-center text-base-content/70"
                    >
                        Adicione pelo menos um campo para uma página.
                    </p>{/if}
            </fieldset>

            <!-- Permissions Fieldset -->
            <fieldset class="border p-4 rounded-md border-base-content/20">
                <legend class="text-lg font-semibold px-2"
                    >Permissões</legend
                >
                <div class="overflow-x-auto">
                    <table class="table table-sm w-full">
                        <thead>
                            <tr>
                                <th>Função</th> <th class="text-center">Ver</th>
                                <th class="text-center">Criar</th>
                                <th class="text-center">Editar</th>
                                <th class="text-center">Eliminar</th>
                                <th class="text-center">Gerir Campos</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each roles as role (role.id)}
                                {@const perm = permissions[role.id]}
                                {#if perm}
                                    <tr>
                                        <td class="font-medium"
                                            >{role.name}
                                            {#if role.is_admin}(Admin){/if}</td
                                        >
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        perm.can_view ||
                                                        role.is_admin,
                                                    (value) =>
                                                        (perm.can_view = value)
                                                }
                                                disabled={role.is_admin}
                                            /></td
                                        >
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        perm.can_create ||
                                                        role.is_admin,
                                                    (value) =>
                                                        (perm.can_create =
                                                            value)
                                                }
                                                disabled={role.is_admin}
                                            /></td
                                        >
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        perm.can_edit ||
                                                        role.is_admin,
                                                    (value) =>
                                                        (perm.can_edit = value)
                                                }
                                                disabled={role.is_admin}
                                            /></td
                                        >
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        perm.can_delete ||
                                                        role.is_admin,
                                                    (value) =>
                                                        (perm.can_delete =
                                                            value)
                                                }
                                                disabled={role.is_admin}
                                            /></td
                                        >
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        perm.can_manage_fields ||
                                                        role.is_admin,
                                                    (value) =>
                                                        (perm.can_manage_fields =
                                                            value)
                                                }
                                                disabled={role.is_admin}
                                            /></td
                                        >
                                    </tr>
                                {/if}
                            {/each}
                        </tbody>
                    </table>
                </div>
                {#if roles.length === 0}<p
                        class="text-center text-base-content/70"
                    >
                        Nenhuma função encontrada. Crie funções primeiro.
                    </p>{/if}
            </fieldset>
        {/if}

        <!-- Actions -->
        <div class="flex justify-end gap-4">
            <a href="/admin/pages/" class="btn btn-ghost">Cancelar</a>
            <button
                type="submit"
                class="btn btn-primary"
                disabled={isSubmitting || isLoading}
            >
                {#if isSubmitting}<span
                        class="loading loading-spinner loading-sm"
                    ></span> Guardando...{:else}Criar {pageData.is_group
                        ? "Grupo"
                        : "Página"}{/if}
            </button>
        </div>
    </form>
{/if}
