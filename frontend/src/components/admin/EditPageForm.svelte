<script lang="ts">
    import { onMount, tick } from "svelte";
    // ... other imports
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    import FieldOptionsEditor from "./FieldOptionsEditor.svelte"; // Import the new component
    import {
        getCustomPageById,
        updateCustomPage,
        updatePagePermissions,
    } from "@api/custom-pages-api";
    import {
        addPageField,
        deleteField,
        getFieldTypes,
        getValidations,
        updateField,
    } from "@api/fields-api";
    import { getRoles } from "@api/roles-api";
    import type { Role } from "@lib/types/roles";
    import type {
        CreatePageFieldRequest,
        FieldType as BackendFieldType, // Rename to avoid clash if needed
        PageField,
        UpdatePageFieldRequest, // Import UpdatePageFieldRequest
        ValidationFunction,
    } from "@lib/types/fields";
    import type {
        RolePermissionRequest,
        UpdateCustomPageRequest,
    } from "@lib/types/custom-page";

    const { pageId }: { pageId: number } = $props();

    // --- State ---
    // ... (pageData, fields, permissions etc. - same structure as Create form)
    let pageData = $state<Partial<UpdateCustomPageRequest>>({});
    let isGroup = $state(false);
    let originalPageDataJson = $state("");
    // Define a type for the fields array with transient properties
    type FormFieldWithState = PageField & {
        isNew?: boolean;
        isDeleted?: boolean;
    };
    let fields = $state<FormFieldWithState[]>([]); // Use the extended type
    let originalFieldsJson = $state("");
    let permissions = $state<Record<number, RolePermissionRequest>>({});
    let originalPermissionsJson = $state("");
    let fieldTypes = $state<BackendFieldType[]>([]);
    let validations = $state<ValidationFunction[]>([]);
    let roles = $state<Role[]>([]);
    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});
    let pagePath = $state("");

    const fieldTypeTranslations: Record<string, string> = {
        TEXT: "Texto Curto",
        NUMBER: "Número",
        SELECT: "Seleção Única",
        DATE: "Data",
        DATE_RANGE: "Intervalo de Datas",
        TEXTAREA: "Texto Longo",
    };

    // --- Fetch Initial Data ---
    onMount(async () => {
        // ... (fetching logic remains the same, populating state from fetchedPageData)
        try {
            const [
                fetchedFieldTypes,
                fetchedValidations,
                fetchedRoles,
                fetchedPageData,
            ] = await Promise.all([
                getFieldTypes(),
                getValidations(),
                getRoles(),
                getCustomPageById(pageId),
            ]);
            fieldTypes = fetchedFieldTypes;
            validations = fetchedValidations;
            roles = fetchedRoles;
            if (!fetchedPageData) throw new Error("Página não encontrada.");

            pageData = {
                name: fetchedPageData.page.name,
                parent_path: fetchedPageData.page.parent_path,
                description: fetchedPageData.page.description,
                icon: fetchedPageData.page.icon,
                notify_on_new_record: fetchedPageData.page.notify_on_new_record,
                requires_acknowledgment:
                    fetchedPageData.page.requires_acknowledgment,
            };
            isGroup = fetchedPageData.page.is_group;
            originalPageDataJson = JSON.stringify(pageData);
            pagePath = fetchedPageData.page.path;

            if (!isGroup) {
                // IMPORTANT: Initialize fields, including notification fields
                fields = fetchedPageData.fields.map((f) => ({
                    ...f,
                    options: f.options ?? null,
                    validation_name: f.validation_name ?? null,
                    // Ensure notification fields are initialized correctly
                    notification_enabled: f.notification_enabled ?? false,
                    notification_days_before:
                        f.notification_days_before ?? null,
                    notification_target_date_part:
                        f.notification_target_date_part ?? null,
                    isNew: false,
                    isDeleted: false,
                }));
                // Filter out transient props before stringifying for comparison
                originalFieldsJson = JSON.stringify(
                    fields.map(({ id, isNew, isDeleted, ...rest }) => ({
                        id,
                        ...rest,
                    })), // Keep ID for matching
                );

                const initialPermissions: Record<
                    number,
                    RolePermissionRequest
                > = {};
                roles.forEach((role) => {
                    const existingPerm = fetchedPageData.permissions.find(
                        (p) => p.role_id === role.id,
                    );
                    initialPermissions[role.id] = existingPerm
                        ? {
                              role_id: role.id,
                              can_view: existingPerm.can_view,
                              can_create: existingPerm.can_create,
                              can_edit: existingPerm.can_edit,
                              can_delete: existingPerm.can_delete,
                              can_manage_fields: existingPerm.can_manage_fields,
                              can_view_acknowledgments:
                                  (existingPerm as any)
                                      .can_view_acknowledgments || false, // Load if exists
                          }
                        : {
                              role_id: role.id,
                              can_view: false,
                              can_create: false,
                              can_edit: false,
                              can_delete: false,
                              can_manage_fields: false,
                              can_view_acknowledgments: false, // Default for new
                          };
                });
                permissions = initialPermissions;
                originalPermissionsJson = JSON.stringify(
                    Object.values(permissions),
                );
            } else {
                fields = [];
                originalFieldsJson = JSON.stringify([]);
                permissions = {};
                originalPermissionsJson = JSON.stringify([]);
            }
        } catch (e: any) {
            showAlert(
                `Erro ao carregar dados da página: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isLoading = false;
        }
    });

    // --- Field Management (identical to Create form) ---
    function addField() {
        // ... (same as Create form, ensure options: null)
        if (isGroup) return;
        fields.push({
            id: undefined,
            isNew: true,
            isDeleted: false,
            name: `novo_campo_${fields.length + 1}`,
            display_name: "",
            field_type_id: fieldTypes[0]?.id || 1,
            required: false,
            options: null,
            validation_name: null,
            is_searchable: true,
            is_displayed_in_table: true,
            order_index: fields.filter((f) => !f.isDeleted).length,
            notification_enabled: false, // Initialize notification fields for new fields
            notification_days_before: null,
            notification_target_date_part: null,
        });
        fields = [...fields];
        updateOrderIndexes();
    }
    function removeField(index: number) {
        // ... (same as Create form)
        if (isGroup) return;
        if (fields[index].isNew) fields.splice(index, 1);
        else fields[index].isDeleted = true;
        fields = [...fields];
        updateOrderIndexes();
    }
    function undeleteField(index: number) {
        // ... (same as Create form)
        if (isGroup) return;
        fields[index].isDeleted = false;
        fields = [...fields];
        updateOrderIndexes();
    }
    function updateOrderIndexes() {
        if (isGroup) return;
        let currentOrder = 0;
        fields.forEach((field) => {
            if (!field.isDeleted) field.order_index = currentOrder++;
        });
        fields = [...fields];
    }
    function handleFieldNameChange(index: number, event: Event) {
        // ... (same as Create form)
        if (isGroup || !fields[index].isNew) return;
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

    function getTranslatedFieldTypeName(
        backendName: string | undefined,
    ): string {
        if (!backendName) return "Desconhecido";
        return fieldTypeTranslations[backendName] || backendName;
    }

    // Helper to get field type object
    function getFieldTypeById(
        fieldTypeId: number,
    ): BackendFieldType | undefined {
        return fieldTypes.find((ft) => ft.id === fieldTypeId);
    }
    // --- REMOVE parseOptions ---

    // --- Form Submission ---
    async function handleSubmit(e: Event) {
        e.preventDefault();
        isSubmitting = true;
        errors = {};

        // --- Validation ---
        if (!pageData.name) errors["page_name"] = "Nome obrigatório.";
        if (!isGroup) {
            const activeFields = fields.filter((f) => !f.isDeleted);
            activeFields.forEach((field, index) => {
                const errorPrefix = `field_${field.id ?? `new_${index}`}`;
                if (!field.display_name)
                    errors[`${errorPrefix}_display_name`] = "Obrigatório.";
                if (!field.name) errors[`${errorPrefix}_name`] = "Obrigatório.";
                else if (!/^[a-z0-9_]+$/.test(field.name))
                    errors[`${errorPrefix}_name`] = "Inválido.";
                // Options validation handled by editor
            });
            if (activeFields.length === 0)
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
        // Format parent path
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
        const finalPageData = { ...pageData, parent_path: formattedParentPath };

        const pageDetailsChanged =
            JSON.stringify(finalPageData) !== originalPageDataJson;
        const currentPermissions = Object.values(permissions);
        const permissionsChanged =
            !isGroup &&
            JSON.stringify(currentPermissions) !== originalPermissionsJson;

        const fieldsToCreate: CreatePageFieldRequest[] = [];
        // Use the imported UpdatePageFieldRequest directly
        const fieldsToUpdate: Array<{ id: number } & UpdatePageFieldRequest> =
            [];
        const fieldsToDelete: number[] = [];
        let fieldsChanged = false;

        if (!isGroup) {
            // Compare current fields (excluding transient props) with original snapshot
            const currentFieldsComparable = JSON.stringify(
                fields
                    .filter((f) => !f.isDeleted && !f.isNew)
                    .map(({ id, isNew, isDeleted, ...rest }) => ({
                        id,
                        ...rest,
                    })),
            );
            // Compare original fields (filtered to exclude potential transient props if added later)
            const originalFieldsComparable = JSON.stringify(
                JSON.parse(originalFieldsJson).map(
                    ({ id, isNew, isDeleted, ...rest }: any) => ({
                        id,
                        ...rest,
                    }),
                ),
            );

            if (
                currentFieldsComparable !== originalFieldsComparable ||
                fields.some((f) => f.isNew && !f.isDeleted) ||
                fields.some((f) => f.isDeleted && !f.isNew)
            ) {
                fieldsChanged = true;
                fields.forEach((field) => {
                    if (field.isDeleted && !field.isNew && field.id) {
                        fieldsToDelete.push(field.id);
                    } else if (!field.isDeleted) {
                        const { id, isNew, isDeleted, ...fieldPayload } = field;
                        // Ensure options is null if empty array before sending
                        const payloadWithOptions = {
                            ...fieldPayload,
                            options: fieldPayload.options ?? null, // Ensure null if needed
                            // Ensure notification fields are null if disabled
                            notification_days_before:
                                fieldPayload.notification_enabled
                                    ? fieldPayload.notification_days_before
                                    : null,
                            notification_target_date_part:
                                fieldPayload.notification_enabled
                                    ? fieldPayload.notification_target_date_part
                                    : null,
                        };

                        if (isNew) {
                            // Use the full payload including name for creation
                            fieldsToCreate.push(payloadWithOptions);
                        } else if (id) {
                            // Find the original field data (without transient props)
                            const originalField = JSON.parse(
                                originalFieldsJson,
                            ).find((of: any) => of.id === id);

                            // Prepare the update payload (exclude ID, name, isNew, isDeleted)
                            // NOTE: updatePayload now contains the correctly nulled notification fields if needed
                            const {
                                name: _name, // Name cannot be updated
                                ...updatePayload
                            } = payloadWithOptions; // Use the correct variable name here

                            // Compare the update payload with the original field data
                            // Need to map original to also nullify notification fields if disabled for fair compare
                            const {
                                id: _id,
                                isNew: _isNew,
                                isDeleted: _isDeleted,
                                ...originalRawComparable
                            } = originalField || {};

                            // Nullify original notification fields if disabled for comparison
                            const originalComparable = {
                                ...originalRawComparable,
                                notification_days_before:
                                    originalRawComparable.notification_enabled
                                        ? originalRawComparable.notification_days_before
                                        : null,
                                notification_target_date_part:
                                    originalRawComparable.notification_enabled
                                        ? originalRawComparable.notification_target_date_part
                                        : null,
                            };

                            if (
                                originalField &&
                                JSON.stringify(updatePayload) !==
                                    JSON.stringify(originalComparable)
                            ) {
                                // Only add to update if something actually changed
                                // Ensure the payload matches UpdatePageFieldRequest
                                fieldsToUpdate.push({
                                    id,
                                    ...(updatePayload as UpdatePageFieldRequest),
                                });
                            }
                        }
                    }
                });
            }
        }
        // --- End Prepare Data ---

        if (!pageDetailsChanged && !permissionsChanged && !fieldsChanged) {
            showAlert(
                "Nenhuma alteração detetada.",
                AlertType.INFO,
                AlertPosition.TOP,
            );
            isSubmitting = false;
            return;
        }

        // --- API Calls (same logic as Create form, using update/add/delete field APIs) ---
        try {
            // ... (Promise.all logic for updateCustomPage, updatePagePermissions, deleteField, updateField, addPageField) ...
            const promises: Promise<any>[] = [];
            if (pageDetailsChanged)
                promises.push(
                    updateCustomPage(
                        pageId,
                        finalPageData as UpdateCustomPageRequest,
                    ).then((ok) => {
                        if (!ok)
                            throw new Error("Falha ao atualizar detalhes.");
                    }),
                );
            if (!isGroup) {
                if (permissionsChanged)
                    promises.push(
                        updatePagePermissions(pageId, currentPermissions).then(
                            (ok) => {
                                if (!ok)
                                    throw new Error(
                                        "Falha ao atualizar permissões.",
                                    );
                            },
                        ),
                    );
                fieldsToDelete.forEach((id) =>
                    promises.push(
                        deleteField(id).then((ok) => {
                            if (!ok)
                                throw new Error(
                                    `Falha ao eliminar campo ${id}.`,
                                );
                        }),
                    ),
                );
                fieldsToUpdate.forEach((f) =>
                    promises.push(
                        // Pass UpdatePageFieldRequest type here
                        updateField(f.id, f as UpdatePageFieldRequest).then(
                            (ok) => {
                                if (!ok)
                                    throw new Error(
                                        `Falha ao atualizar campo ${f.display_name} (ID: ${f.id}).`,
                                    );
                            },
                        ),
                    ),
                );
                fieldsToCreate.forEach((f) =>
                    promises.push(
                        addPageField(pageId, f).then((res) => {
                            if (!res.success)
                                throw new Error(
                                    `Falha ao criar campo ${f.display_name}.`,
                                );
                        }),
                    ),
                );
            }
            await Promise.all(promises);
            showAlert(
                `${isGroup ? "Grupo" : "Página"} atualizado com sucesso!`,
                AlertType.SUCCESS,
                AlertPosition.TOP,
            );
            if (typeof window !== "undefined")
                window.location.href = "/admin/pages/";
        } catch (e: any) {
            showAlert(
                `Erro ao atualizar ${isGroup ? "grupo" : "página"}: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmitting = false;
        }
        // --- End API Calls ---
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
                >Detalhes {isGroup ? "do Grupo" : "da Página"}</legend
            >

            <div class="form-control w-full">
                <div class="label"><span class="label-text">Tipo</span></div>
                <input
                    type="text"
                    class="input input-bordered w-full bg-base-200"
                    value={isGroup ? "Grupo / Pasta" : "Página de Registos"}
                    readonly
                    disabled
                />
            </div>
            <div class="form-control w-full">
                <div class="label">
                    <span class="label-text">Caminho (URL)</span>
                </div>
                <input
                    type="text"
                    class="input input-bordered w-full bg-base-200"
                    value={pagePath}
                    readonly
                    disabled
                />
            </div>
            <label class="form-control w-full">
                <div class="label"><span class="label-text">Nome*</span></div>
                <input
                    type="text"
                    placeholder={isGroup
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

            <!-- Requires Acknowledgment Checkbox was here, but the correct ones are at the end of this fieldset -->
            <!-- The "Notify on New Record" that was here is the duplicate being removed. -->
            <!-- The actual "Requires Acknowledgment" toggle is now correctly placed at the end of this fieldset,
                 inside the {#if !isGroup} block, along with the non-duplicate "Notify on New Record".
                 However, if "Requires Acknowledgment" was ALSO duplicated before "Description",
                 this edit only removes the "Notify on New Record".
                 Based on user request, only one "Notify on New Record" needs to be removed.
                 The "Requires Acknowledgment" that might have been here is assumed to be handled
                 by the existing correct placement at the end of fieldset.
            -->
            <!-- The div for "Requires Acknowledgment" that was previously here (if it existed as a duplicate)
                 would be part of the old_text if it also needed removal.
                 Given the user asked to remove only one "notify" checkbox, this edit is targeted.
            -->
            <div class="form-control">
                <label class="label cursor-pointer justify-start gap-2">
                    <input
                        type="checkbox"
                        class="toggle toggle-info"
                        bind:checked={pageData.requires_acknowledgment}
                        disabled={isGroup}
                    />
                    <span class="label-text font-medium"
                        >Exigir Tomar Conhecimento?</span
                    >
                </label>
                <div class="label">
                    <span class="label-text-alt"
                        >Se marcado, utilizadores terão de confirmar a leitura
                        antes de ver detalhes de um registo.</span
                    >
                </div>
            </div>
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

            <!-- Notify on New Record Checkbox (Only if NOT a group) -->
            {#if !isGroup}
                <div class="form-control md:col-span-1">
                    <label
                        class="label cursor-pointer justify-start gap-2 pt-8"
                    >
                        <input
                            type="checkbox"
                            class="toggle toggle-info"
                            bind:checked={pageData.notify_on_new_record}
                        />
                        <span class="label-text font-medium"
                            >Notificar em Novos Registos?</span
                        >
                    </label>
                    <div class="label pt-0">
                        <span class="label-text-alt"
                            >Notifica utilizadores com acesso à página quando um
                            novo registo é criado.</span
                        >
                    </div>
                </div>
                <!-- Placeholder for alignment if it's a group or if the other toggle isn't shown -->
                <div class="md:col-span-1"></div>
            {:else}
                <!-- Span two columns if it's a group to maintain layout, or if you want to hide the new record toggle for groups -->
                <div class="md:col-span-2"></div>
            {/if}
        </fieldset>

        {#if !isGroup}
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

                {#each fields as field, index (field.id ?? `new_${index}`)}
                    <div
                        class:border-error={field.isDeleted}
                        class:opacity-60={field.isDeleted}
                        class="border p-3 rounded bg-base-200 relative transition-all"
                    >
                        {#if field.isDeleted}
                            <button
                                type="button"
                                class="btn btn-xs btn-ghost absolute top-2 right-2"
                                title="Restaurar Campo"
                                onclick={() => undeleteField(index)}
                                ><i class="fa-solid fa-undo"></i></button
                            >
                        {:else}
                            <button
                                type="button"
                                class="btn btn-xs btn-error absolute top-2 right-2"
                                title="Remover Campo"
                                onclick={() => removeField(index)}>✕</button
                            >
                        {/if}

                        <div
                            class="grid grid-cols-1 md:grid-cols-3 gap-3"
                            class:pointer-events-none={field.isDeleted}
                        >
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
                                />
                                {#if errors[`field_${field.id ?? `new_${index}`}_display_name`]}<span
                                        class="text-error text-xs mt-1"
                                        >{errors[
                                            `field_${field.id ?? `new_${index}`}_display_name`
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
                                    disabled={!field.isNew}
                                />
                                {#if errors[`field_${field.id ?? `new_${index}`}_name`]}<span
                                        class="text-error text-xs mt-1"
                                        >{errors[
                                            `field_${field.id ?? `new_${index}`}_name`
                                        ]}</span
                                    >{/if}
                                <div class="label">
                                    <span class="label-text-alt"
                                        >{field.isNew
                                            ? "Gerado automaticamente"
                                            : "Não pode ser alterado"}</span
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
                                        fields[index].notification_enabled =
                                            false;
                                        fields[index].notification_days_before =
                                            null;
                                        fields[
                                            index
                                        ].notification_target_date_part = null;
                                        fields = [...fields]; // Trigger reactivity
                                    }}
                                >
                                    {#each fieldTypes as ft (ft.id)}
                                        <option value={ft.id}
                                            >{getTranslatedFieldTypeName(
                                                ft.name,
                                            )}</option
                                        >
                                    {/each}
                                </select>
                            </label>

                            {#if getFieldTypeName(field.field_type_id) === "SELECT"}
                                <div class="md:col-span-3">
                                    <FieldOptionsEditor
                                        bind:optionsJson={field.options}
                                        {fieldTypes}
                                        fieldTypeId={field.field_type_id}
                                    />
                                    {#if errors[`field_${field.id ?? `new_${index}`}_options`]}<span
                                            class="text-error text-xs mt-1"
                                            >{errors[
                                                `field_${field.id ?? `new_${index}`}_options`
                                            ]}</span
                                        >{/if}
                                </div>
                            {/if}

                            <!-- Notification Settings for Date Range -->
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
                                                bind:checked={
                                                    field.notification_enabled
                                                }
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
                                                bind:value={
                                                    field.notification_days_before
                                                }
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
                                                bind:value={
                                                    field.notification_target_date_part
                                                }
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

                            <!-- Validation -->
                            {#if getFieldTypeName(field.field_type_id) === "TEXT"}
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
                                        {#each validations as v (v.name)}
                                            <option value={v.name}
                                                >{v.name} ({v.description})</option
                                            >
                                        {/each}
                                    </select>
                                </label>
                            {:else}
                                <!-- Placeholder for grid alignment if validation is not shown -->
                                <div class="form-control w-full"></div>
                            {/if}

                            <!-- Checkboxes -->
                            <div class="form-control">
                                <label
                                    class="label cursor-pointer justify-start gap-2"
                                    ><input
                                        type="checkbox"
                                        class="checkbox checkbox-sm"
                                        bind:checked={field.required}
                                    /><span class="label-text">Obrigatório</span
                                    ></label
                                >
                            </div>
                            <div class="form-control">
                                <label
                                    class="label cursor-pointer justify-start gap-2"
                                    ><input
                                        type="checkbox"
                                        class="checkbox checkbox-sm"
                                        bind:checked={field.is_searchable}
                                    /><span class="label-text">Pesquisável</span
                                    ></label
                                >
                            </div>
                            <div class="form-control">
                                <label
                                    class="label cursor-pointer justify-start gap-2"
                                    ><input
                                        type="checkbox"
                                        class="checkbox checkbox-sm"
                                        bind:checked={
                                            field.is_displayed_in_table
                                        }
                                    /><span class="label-text"
                                        >Mostrar na Tabela</span
                                    ></label
                                >
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
                {#if fields.filter((f) => !f.isDeleted).length === 0}<p
                        class="text-center text-base-content/70"
                    >
                        Adicione pelo menos um campo para uma página.
                    </p>{/if}
            </fieldset>

            <!-- Permissions Fieldset -->
            <fieldset class="border p-4 rounded-md border-base-content/20">
                <legend class="text-lg font-semibold px-2">Permissões</legend>
                <div class="overflow-x-auto">
                    <table class="table table-sm w-full">
                        <thead>
                            <tr>
                                <th>Função</th> <th class="text-center">Ver</th>
                                <th class="text-center">Criar</th>
                                <th class="text-center">Editar</th>
                                <th class="text-center">Eliminar</th>
                                <th class="text-center">Gerir Campos</th>
                                <th class="text-center">Ver Confirmações</th>
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
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        (perm as any)
                                                            .can_view_acknowledgments ||
                                                        role.is_admin,
                                                    (value) =>
                                                        ((
                                                            perm as any
                                                        ).can_view_acknowledgments =
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
                        Nenhuma função encontrada.
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
                    ></span> Atualizando...{:else}Guardar Alterações{/if}
            </button>
        </div>
    </form>
{/if}
