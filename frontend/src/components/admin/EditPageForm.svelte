<script lang="ts">
    import { onMount, tick } from "svelte";
    import type {
        CustomPageWithFields,
        UpdateCustomPageRequest,
        CreatePageFieldRequest,
        RolePermissionRequest,
    } from "@lib/types/custom-page";
    import type { FieldType as BackendFieldType } from "@lib/types/fields";
    import type { ValidationFunction } from "@lib/types/fields";
    import type { Role } from "@lib/types/roles";
    import {
        getFieldTypes,
        getValidations,
        addPageField,
        updateField,
        deleteField,
    } from "@api/fields-api"; // Add field management APIs
    import { getRoles } from "@api/roles-api";
    import {
        getCustomPageById,
        updateCustomPage,
        updatePagePermissions,
    } from "@api/custom-pages-api"; // Use get by ID
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    const { pageId }: { pageId: number } = $props();

    // --- State ---
    let pageData = $state<Partial<UpdateCustomPageRequest>>({
        name: "",
        description: null,
        icon: null,
    });
    let originalPageDataJson = $state(""); // For basic page details diff
    let fields = $state<
        Array<
            CreatePageFieldRequest & {
                id?: number;
                isNew?: boolean;
                isDeleted?: boolean;
            }
        >
    >([]); // Track existing/new/deleted fields
    let originalFieldsJson = $state(""); // For fields diff
    let permissions = $state<Record<number, RolePermissionRequest>>({}); // Use role ID as key
    let originalPermissionsJson = $state(""); // For permissions diff

    let fieldTypes = $state<BackendFieldType[]>([]);
    let validations = $state<ValidationFunction[]>([]);
    let roles = $state<Role[]>([]);

    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});
    let pagePath = $state(""); // Store original path (cannot be edited)

    // --- Fetch Initial Data ---
    onMount(async () => {
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

            if (!fetchedPageData) {
                throw new Error("Página não encontrada.");
            }

            // Populate state from fetched data
            pageData = {
                name: fetchedPageData.page.name,
                description: fetchedPageData.page.description,
                icon: fetchedPageData.page.icon,
            };
            originalPageDataJson = JSON.stringify(pageData);
            pagePath = fetchedPageData.page.path; // Store original path

            fields = fetchedPageData.fields.map((f) => ({
                ...f,
                isNew: false,
                isDeleted: false,
            }));
            originalFieldsJson = JSON.stringify(
                fields.map(({ id, isNew, isDeleted, ...rest }) => rest),
            ); // Store comparable fields state

            const initialPermissions: Record<number, RolePermissionRequest> =
                {};
            roles.forEach((role) => {
                const existingPerm = fetchedPageData.permissions.find(
                    (p) => p.role_id === role.id,
                );
                initialPermissions[role.id] = existingPerm
                    ? {
                          // Use existing if found
                          role_id: role.id,
                          can_view: existingPerm.can_view,
                          can_create: existingPerm.can_create,
                          can_edit: existingPerm.can_edit,
                          can_delete: existingPerm.can_delete,
                          can_manage_fields: existingPerm.can_manage_fields,
                      }
                    : {
                          // Default if not found for this role
                          role_id: role.id,
                          can_view: false,
                          can_create: false,
                          can_edit: false,
                          can_delete: false,
                          can_manage_fields: false,
                      };
            });
            permissions = initialPermissions;
            originalPermissionsJson = JSON.stringify(
                Object.values(permissions),
            );
        } catch (e: any) {
            showAlert(
                `Erro ao carregar dados da página: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            // Optionally redirect back if page load fails
            // if (typeof window !== 'undefined') window.location.href = '/admin/pages/';
        } finally {
            isLoading = false;
        }
    });

    // --- Field Management (Similar to Create, but handle existing IDs) ---
    function addField() {
        fields.push({
            id: undefined, // No ID yet
            isNew: true, // Mark as new
            isDeleted: false,
            name: `novo_campo_${fields.length + 1}`,
            display_name: "",
            field_type_id: fieldTypes[0]?.id || 1,
            required: false,
            options: null,
            validation_name: null,
            is_searchable: true,
            is_displayed_in_table: true,
            order_index: fields.filter((f) => !f.isDeleted).length, // Order based on non-deleted fields
        });
        fields = [...fields];
        updateOrderIndexes();
    }

    function removeField(index: number) {
        if (fields[index].isNew) {
            fields.splice(index, 1); // Remove new field directly
        } else {
            fields[index].isDeleted = true; // Mark existing field for deletion
        }
        fields = [...fields];
        updateOrderIndexes();
    }

    function undeleteField(index: number) {
        fields[index].isDeleted = false;
        fields = [...fields];
        updateOrderIndexes();
    }

    function updateOrderIndexes() {
        let currentOrder = 0;
        fields.forEach((field) => {
            if (!field.isDeleted) {
                field.order_index = currentOrder++;
            }
        });
        fields = [...fields];
    }

    function handleFieldNameChange(index: number, event: Event) {
        // Only allow changing name for NEW fields before saving
        if (!fields[index].isNew) return;
        const input = event.target as HTMLInputElement;
        fields[index].name = input.value
            .toLowerCase()
            .replace(/\s+/g, "_")
            .replace(/[^a-z0-9_]/g, "");
        fields = [...fields];
    }

    // isOptionsVisible and parseOptions remain the same as in CreatePageForm

    function isOptionsVisible(fieldTypeId: number): boolean {
        const type = fieldTypes.find((ft) => ft.id === fieldTypeId);
        return type?.name === "SELECT";
    }

    function parseOptions(index: number, value: string) {
        try {
            const parsed = JSON.parse(value);
            if (
                Array.isArray(parsed) &&
                parsed.every((item) => typeof item === "string")
            ) {
                fields[index].options = parsed;
                delete errors[`field_${index}_options`];
            } else {
                throw new Error("Deve ser um array JSON de strings.");
            }
        } catch (e) {
            fields[index].options = null;
            errors[`field_${index}_options`] =
                'JSON inválido. Exemplo: ["Opção 1", "Opção 2"]';
        }
        fields = [...fields];
        errors = { ...errors };
    }

    // --- Form Submission ---
    async function handleSubmit(e: Event) {
        e.preventDefault();
        isSubmitting = true;
        errors = {};

        // --- Validation (Similar to Create) ---
        if (!pageData.name)
            errors["page_name"] = "Nome da página é obrigatório.";
        const activeFields = fields.filter((f) => !f.isDeleted);
        activeFields.forEach((field, index) => {
            const errorPrefix = `field_${index}`; // Use index for error keys
            if (!field.display_name)
                errors[`${errorPrefix}_display_name`] =
                    "Nome de exibição é obrigatório.";
            if (!field.name)
                errors[`${errorPrefix}_name`] = "Nome interno é obrigatório.";
            else if (!/^[a-z0-9_]+$/.test(field.name))
                errors[`${errorPrefix}_name`] =
                    "Nome interno inválido (use letras minúsculas, números e _).";
            if (isOptionsVisible(field.field_type_id) && !field.options)
                errors[`${errorPrefix}_options`] =
                    "Opções são obrigatórias para o tipo SELECT (use formato JSON).";
        });

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

        // --- Prepare Data for API Calls ---
        const pageDetailsChanged =
            JSON.stringify(pageData) !== originalPageDataJson;
        const currentPermissions = Object.values(permissions);
        const permissionsChanged =
            JSON.stringify(currentPermissions) !== originalPermissionsJson;

        // Separate fields into new, updated, deleted
        const fieldsToCreate: CreatePageFieldRequest[] = [];
        const fieldsToUpdate: Array<
            { id: number } & Omit<CreatePageFieldRequest, "name">
        > = []; // Cannot update 'name'
        const fieldsToDelete: number[] = [];

        fields.forEach((field) => {
            if (field.isDeleted && !field.isNew && field.id) {
                fieldsToDelete.push(field.id);
            } else if (!field.isDeleted) {
                const { id, isNew, isDeleted, ...fieldPayload } = field;
                if (isNew) {
                    fieldsToCreate.push(fieldPayload);
                } else if (id) {
                    // Check if this existing field actually changed
                    const originalField = JSON.parse(originalFieldsJson).find(
                        (of: any) => of.id === id,
                    );
                    const currentFieldComparable = { ...fieldPayload }; // Create comparable version
                    if (
                        JSON.stringify(currentFieldComparable) !==
                        JSON.stringify(originalField)
                    ) {
                        // Cannot update name, remove it from payload
                        const { name, ...updatePayload } = fieldPayload;
                        fieldsToUpdate.push({ id, ...updatePayload });
                    }
                }
            }
        });

        const fieldsChanged =
            fieldsToCreate.length > 0 ||
            fieldsToUpdate.length > 0 ||
            fieldsToDelete.length > 0;

        if (!pageDetailsChanged && !permissionsChanged && !fieldsChanged) {
            showAlert(
                "Nenhuma alteração detetada.",
                AlertType.INFO,
                AlertPosition.TOP,
            );
            isSubmitting = false;
            return;
        }

        // --- Execute API Calls ---
        try {
            let success = true;
            const promises: Promise<any>[] = [];

            // 1. Update Page Details
            if (pageDetailsChanged) {
                promises.push(
                    updateCustomPage(
                        pageId,
                        pageData as UpdateCustomPageRequest,
                    ).then((ok) => {
                        if (!ok)
                            throw new Error(
                                "Falha ao atualizar detalhes da página.",
                            );
                    }),
                );
            }

            // 2. Update Permissions
            if (permissionsChanged) {
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
            }

            // 3. Delete Fields
            fieldsToDelete.forEach((fieldId) => {
                promises.push(
                    deleteField(fieldId).then((ok) => {
                        if (!ok)
                            throw new Error(
                                `Falha ao eliminar campo ${fieldId}.`,
                            );
                    }),
                );
            });

            // 4. Update Fields
            fieldsToUpdate.forEach((fieldUpdate) => {
                promises.push(
                    updateField(fieldUpdate.id, fieldUpdate).then((ok) => {
                        if (!ok)
                            throw new Error(
                                `Falha ao atualizar campo ${fieldUpdate.id}.`,
                            );
                    }),
                );
            });

            // 5. Create Fields
            fieldsToCreate.forEach((fieldCreate) => {
                promises.push(
                    addPageField(pageId, fieldCreate).then((result) => {
                        if (!result.success)
                            throw new Error(
                                `Falha ao criar campo ${fieldCreate.display_name}.`,
                            );
                    }),
                );
            });

            // Wait for all updates
            await Promise.all(promises);

            showAlert(
                "Página atualizada com sucesso!",
                AlertType.SUCCESS,
                AlertPosition.TOP,
            );
            // Optionally redirect back to list
            if (typeof window !== "undefined") {
                window.location.href = "/admin/pages/";
            }
        } catch (e: any) {
            showAlert(
                `Erro ao atualizar página: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmitting = false;
        }
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
        <!--- Page Details (Path is read-only) --->
        <fieldset
            class="grid grid-cols-1 md:grid-cols-2 gap-4 border p-4 rounded-md border-base-content/20"
        >
            <legend class="text-lg font-semibold px-2"
                >Detalhes da Página</legend
            >

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
            <div class="form-control w-full">
                <!-- -- Placeholder to balance grid -->
            </div>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Nome da Página*</span>
                </div>
                <input
                    type="text"
                    placeholder="Ex: Licenças de Software"
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
                    placeholder="Breve descrição da finalidade da página"
                    class="textarea textarea-bordered w-full"
                    bind:value={pageData.description}
                ></textarea>
            </label>
        </fieldset>

        <!--- Fields (with delete/undelete logic) -->
        <fieldset
            class="border p-4 rounded-md border-base-content/20 space-y-4"
        >
            <legend class="text-lg font-semibold px-2">Campos do Registo</legend
            >
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
                        >
                            <i class="fa-solid fa-undo"></i>
                        </button>
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
                        <!-- -- Field Config Inputs (similar to Create) -->
                        <label class="form-control w-full">
                            <div class="label">
                                <span class="label-text">Nome Exibição*</span>
                            </div>
                            <input
                                type="text"
                                placeholder="Ex: Nome da Licença"
                                class="input input-sm input-bordered w-full"
                                bind:value={field.display_name}
                                required
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
                                <span class="label-text">Nome Interno*</span>
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
                            {#if errors[`field_${index}_name`]}<span
                                    class="text-error text-xs mt-1"
                                    >{errors[`field_${index}_name`]}</span
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
                            >
                                {#each fieldTypes as ft}
                                    <option value={ft.id}>{ft.name}</option>
                                {/each}
                            </select>
                        </label>

                        {#if isOptionsVisible(field.field_type_id)}
                            <label class="form-control w-full md:col-span-3">
                                <div class="label">
                                    <span class="label-text"
                                        >Opções (JSON Array)*</span
                                    >
                                </div>
                                <textarea
                                    placeholder="['Opção A', 'Opção B', 'Opção C']"
                                    class="textarea textarea-sm textarea-bordered w-full font-mono"
                                    rows="2"
                                    value={JSON.stringify(field.options) ?? ""}
                                    oninput={(e) =>
                                        parseOptions(
                                            index,
                                            (e.target as HTMLTextAreaElement)
                                                .value,
                                        )}
                                    required
                                ></textarea>
                                {#if errors[`field_${index}_options`]}<span
                                        class="text-error text-xs mt-1"
                                        >{errors[
                                            `field_${index}_options`
                                        ]}</span
                                    >{/if}
                            </label>
                        {/if}

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
                                    bind:checked={field.is_displayed_in_table}
                                />
                                <span class="label-text">Mostrar na Tabela</span
                                >
                            </label>
                        </div>
                        <input type="hidden" bind:value={field.order_index} />
                    </div>
                </div>
            {/each}
            <button
                type="button"
                class="btn btn-sm btn-outline btn-accent"
                onclick={addField}
            >
                <i class="fa-solid fa-plus mr-1"></i> Adicionar Campo
            </button>
        </fieldset>

        <!--- Permissions (Same as Create) --->
        <fieldset class="border p-4 rounded-md border-base-content/20">
            <legend class="text-lg font-semibold px-2"
                >Permissões por Função</legend
            >
            <div class="overflow-x-auto">
                <table class="table table-sm w-full">
                    <thead>
                        <tr>
                            <th>Função</th>
                            <th class="text-center">Ver</th>
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
                                <!-- -- Ensure perm exists -->
                                <tr>
                                    <td class="font-medium"
                                        >{role.name}
                                        {#if role.is_admin}(Admin){/if}</td
                                    >
                                    <td class="text-center"
                                        ><input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            bind:checked={perm.can_view}
                                            disabled={role.is_admin}
                                        /></td
                                    >
                                    <td class="text-center"
                                        ><input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            bind:checked={perm.can_create}
                                            disabled={role.is_admin}
                                        /></td
                                    >
                                    <td class="text-center"
                                        ><input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            bind:checked={perm.can_edit}
                                            disabled={role.is_admin}
                                        /></td
                                    >
                                    <td class="text-center"
                                        ><input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            bind:checked={perm.can_delete}
                                            disabled={role.is_admin}
                                        /></td
                                    >
                                    <td class="text-center"
                                        ><input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            bind:checked={
                                                perm.can_manage_fields
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
            {#if roles.length === 0}
                <p class="text-center text-base-content/70">
                    Nenhuma função encontrada.
                </p>
            {/if}
        </fieldset>

        <div class="flex justify-end gap-4">
            <a href="/admin/pages/" class="btn btn-ghost">Cancelar</a>
            <button
                type="submit"
                class="btn btn-primary"
                disabled={isSubmitting || isLoading}
            >
                {#if isSubmitting}
                    <span class="loading loading-spinner loading-sm"></span> Atualizando...
                {:else}
                    Guardar Alterações
                {/if}
            </button>
        </div>
    </form>
{/if}
