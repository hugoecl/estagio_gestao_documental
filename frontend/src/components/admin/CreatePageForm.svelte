<script lang="ts">
    import { onMount, tick } from "svelte";
    import type {
        CreateCustomPageRequest,
        CreatePageFieldRequest,
        RolePermissionRequest,
    } from "@lib/types/custom-page";
    import type { FieldType as BackendFieldType } from "@lib/types/fields";
    import type { ValidationFunction } from "@lib/types/fields";
    import type { Role } from "@lib/types/roles";
    import { getFieldTypes, getValidations } from "@api/fields-api";
    import { getRoles } from "@api/roles-api";
    import { createCustomPage } from "@api/custom-pages-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    // --- State ---
    let pageData = $state<Partial<CreateCustomPageRequest>>({
        name: "",
        path: "",
        parent_path: null,
        description: null,
        icon: null,
        fields: [],
        permissions: [],
    });
    let fields = $state<CreatePageFieldRequest[]>([]);
    let permissions = $state<Record<number, RolePermissionRequest>>({}); // Use role ID as key

    let fieldTypes = $state<BackendFieldType[]>([]);
    let validations = $state<ValidationFunction[]>([]);
    let roles = $state<Role[]>([]);

    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});

    // --- Fetch Initial Data ---
    onMount(async () => {
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

            // Initialize permissions based on fetched roles
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
        fields.push({
            name: `campo_${fields.length + 1}`, // Auto-generate name initially
            display_name: "",
            field_type_id: fieldTypes[0]?.id || 1, // Default to first type or TEXT
            required: false,
            options: null,
            validation_name: null,
            is_searchable: true,
            is_displayed_in_table: true,
            order_index: fields.length,
        });
        fields = [...fields]; // Trigger reactivity
    }

    function removeField(index: number) {
        fields.splice(index, 1);
        // Re-index remaining fields
        fields.forEach((field, i) => (field.order_index = i));
        fields = [...fields]; // Trigger reactivity
    }

    function handleFieldNameChange(index: number, event: Event) {
        const input = event.target as HTMLInputElement;
        // Basic slugification - replace spaces/special chars with underscores, lowercase
        fields[index].name = input.value
            .toLowerCase()
            .replace(/\s+/g, "_")
            .replace(/[^a-z0-9_]/g, "");
        fields = [...fields];
    }

    function isOptionsVisible(fieldTypeId: number): boolean {
        const type = fieldTypes.find((ft) => ft.id === fieldTypeId);
        return type?.name === "SELECT";
    }

    function parseOptions(index: number, value: string) {
        try {
            // Expect JSON array of strings: ["Option 1", "Option 2"]
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
            fields[index].options = null; // Clear on error
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
        errors = {}; // Clear previous errors

        // Basic Validation
        if (!pageData.name)
            errors["page_name"] = "Nome da página é obrigatório.";
        if (!pageData.path)
            errors["page_path"] = "Caminho da página é obrigatório.";
        else if (!/^[a-z0-9\/-]+$/.test(pageData.path))
            errors["page_path"] =
                "Caminho inválido (use letras minúsculas, números, / e -).";

        fields.forEach((field, index) => {
            if (!field.display_name)
                errors[`field_${index}_display_name`] =
                    "Nome de exibição é obrigatório.";
            if (!field.name)
                errors[`field_${index}_name`] = "Nome interno é obrigatório.";
            else if (!/^[a-z0-9_]+$/.test(field.name))
                errors[`field_${index}_name`] =
                    "Nome interno inválido (use letras minúsculas, números e _).";
            if (isOptionsVisible(field.field_type_id) && !field.options)
                errors[`field_${index}_options`] =
                    "Opções são obrigatórias para o tipo SELECT (use formato JSON).";
        });

        if (Object.keys(errors).length > 0) {
            showAlert(
                "Existem erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            isSubmitting = false;
            errors = { ...errors }; // Trigger reactivity
            return;
        }

        // Format path
        let formattedPath = pageData.path!.trim().toLowerCase();
        if (!formattedPath.startsWith("/")) formattedPath = "/" + formattedPath;
        if (!formattedPath.endsWith("/")) formattedPath += "/";

        let formattedParentPath =
            pageData.parent_path?.trim().toLowerCase() || null;
        if (formattedParentPath) {
            if (!formattedParentPath.startsWith("/"))
                formattedParentPath = "/" + formattedParentPath;
            if (!formattedParentPath.endsWith("/")) formattedParentPath += "/";
        }

        const finalData: CreateCustomPageRequest = {
            name: pageData.name!,
            path: formattedPath,
            parent_path: formattedParentPath,
            description: pageData.description || null,
            icon: pageData.icon || null,
            fields: fields,
            permissions: Object.values(permissions), // Convert permission record to array
        };

        try {
            const result = await createCustomPage(finalData);
            if (result.success) {
                showAlert(
                    "Página criada com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                // Redirect to the new page or the admin list
                if (typeof window !== "undefined") {
                    window.location.href = "/admin/pages/"; // Redirect to list
                }
            } else {
                throw new Error("Falha ao criar página no backend.");
            }
        } catch (e: any) {
            showAlert(
                `Erro ao criar página: ${e.message}`,
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
        <!-- Page Details -->
        <fieldset
            class="grid grid-cols-1 md:grid-cols-2 gap-4 border p-4 rounded-md border-base-content/20"
        >
            <legend class="text-lg font-semibold px-2"
                >Detalhes da Página</legend
            >

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
                    <span class="label-text">Caminho (URL)*</span>
                </div>
                <input
                    type="text"
                    placeholder="Ex: /gestao/licencas-software"
                    class="input input-bordered w-full"
                    bind:value={pageData.path}
                    required
                />
                {#if errors.page_path}<span class="text-error text-xs mt-1"
                        >{errors.page_path}</span
                    >{/if}
                <div class="label">
                    <span class="label-text-alt"
                        >Será formatado para /caminho/ (minúsculas, -, /)</span
                    >
                </div>
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Caminho Pai (Opcional)</span>
                </div>
                <input
                    type="text"
                    placeholder="Ex: /gestao/ ou deixe em branco"
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
                    placeholder="Breve descrição da finalidade da página"
                    class="textarea textarea-bordered w-full"
                    bind:value={pageData.description}
                ></textarea>
            </label>
        </fieldset>

        <!--- Fields -->
        <fieldset
            class="border p-4 rounded-md border-base-content/20 space-y-4"
        >
            <legend class="text-lg font-semibold px-2">Campos do Registo</legend
            >
            {#each fields as field, index (index)}
                <div class="border p-3 rounded bg-base-200 relative">
                    <button
                        type="button"
                        class="btn btn-xs btn-circle btn-error absolute top-2 right-2"
                        title="Remover Campo"
                        onclick={() => removeField(index)}>✕</button
                    >
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <!-- Field Config -->
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
                                oninput={(e) => handleFieldNameChange(index, e)}
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
                                class="input input-sm input-bordered w-full"
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
                        <!-- Store order -->
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
            {#if fields.length === 0}
                <p class="text-center text-base-content/70">
                    Adicione pelo menos um campo.
                </p>
            {/if}
        </fieldset>

        <!-- Permissions -->
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
                                        bind:checked={perm.can_manage_fields}
                                        disabled={role.is_admin}
                                    /></td
                                >
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            {#if roles.length === 0}
                <p class="text-center text-base-content/70">
                    Nenhuma função encontrada. Crie funções primeiro.
                </p>
            {/if}
        </fieldset>

        <!-- Actions -->
        <div class="flex justify-end gap-4">
            <a href="/admin/pages/" class="btn btn-ghost">Cancelar</a>
            <button
                type="submit"
                class="btn btn-primary"
                disabled={isSubmitting || isLoading}
            >
                {#if isSubmitting}
                    <span class="loading loading-spinner loading-sm"></span> Guardando...
                {:else}
                    Criar Página
                {/if}
            </button>
        </div>
    </form>
{/if}
