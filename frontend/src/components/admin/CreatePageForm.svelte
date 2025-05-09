<script lang="ts">
    import { onMount, tick } from "svelte";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    import FieldOptionsEditor from "./FieldOptionsEditor.svelte";
    import { getFieldTypes, getValidations } from "@api/fields-api";
    import { getRoles } from "@api/roles-api";
    import type {
        CreateCustomPageRequest,
        RolePermissionRequest,
    } from "@lib/types/custom-page";
    import type { Role } from "@lib/types/roles";
    import type {
        FieldType as BackendFieldType, // Renamed to avoid conflict with local FieldType if any
        CreatePageFieldRequest,
        ValidationFunction,
    } from "@lib/types/fields";
    import { createCustomPage, getGroupPages, type CustomPage } from "@api/custom-pages-api"; // Added getGroupPages and CustomPage
    import { toSearchString } from "@utils/search-utils"; // Added for path generation

    // --- State ---
    let pageData = $state<Partial<CreateCustomPageRequest>>({
        name: "",
        path: "/", // Default to root, will be auto-generated
        parent_path: null, // Will be set by selectedParentPath
        is_group: false,
        description: "",
        icon: "",
        notify_on_new_record: false,
        requires_acknowledgment: false,
    });
    let fields = $state<Array<CreatePageFieldRequest & { key: Symbol }>>([]);
    let permissions = $state<Record<number, RolePermissionRequest>>({});
    let fieldTypes = $state<BackendFieldType[]>([]);
    let validations = $state<ValidationFunction[]>([]);
    let roles = $state<Role[]>([]);
    let availableGroups = $state<CustomPage[]>([]); // For parent group selector
    let selectedParentGroupId = $state<string | null>(null); // Store ID of selected parent group
    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});


    // --- Path Generation ---
    function generatePathFromName(name: string, parentPath: string | null): string {
        let slug = toSearchString(name).replace(/\s+/g, "-"); // Replace spaces with hyphens
        slug = slug.replace(/[^a-z0-9-]/g, ""); // Remove invalid chars, keep hyphens
        slug = slug.replace(/-+/g, "-"); // Replace multiple hyphens with single
        slug = slug.trimStart().trimEnd(); // Trim leading/trailing spaces/hyphens (though regex should handle most)
        if (!slug) { // Handle empty slug after processing
            slug = "nova-pagina"; // Default slug if name results in empty
        }

        if (parentPath && parentPath !== "/") {
            const cleanParentPath = parentPath.endsWith("/") ? parentPath.slice(0, -1) : parentPath;
            return `${cleanParentPath}/${slug}`;
        }
        return `/${slug}`;
    }

    $effect(() => {
        const selectedGroup = availableGroups.find(g => g.id.toString() === selectedParentGroupId);
        const parentPath = selectedGroup ? selectedGroup.path : null;
        pageData.parent_path = parentPath; // Update parent_path for submission
        pageData.path = generatePathFromName(pageData.name || "", parentPath);
    });


    // --- Fetch Initial Data ---
    onMount(async () => {
        try {
            const [
                fetchedFieldTypes,
                fetchedValidations,
                fetchedRoles,
                fetchedGroups, // Fetch groups
            ] = await Promise.all([
                getFieldTypes(),
                getValidations(),
                getRoles(),
                getGroupPages(), // Call new API function
            ]);
            fieldTypes = fetchedFieldTypes;
            availableGroups = fetchedGroups; // Store groups
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
                    can_view_acknowledgments: false, // Added
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
            key: Symbol(), // Unique key for #each block
            name: `campo_${fields.length + 1}`,
            display_name: "",
            field_type_id: fieldTypes[0]?.id || 1,
            required: false,
            options: null,
            validation_name: null,
            is_searchable: true,
            is_displayed_in_table: true,
            order_index: fields.length,
            notification_enabled: false,
            notification_days_before: null,
            notification_target_date_part: null,
        });
        fields = [...fields];
    }

    function removeField(index: number) {
        fields.splice(index, 1);
        fields.forEach((field, i) => (field.order_index = i));
        fields = [...fields];
    }

    function handleFieldNameChange(index: number, event: Event) {
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
    function getFieldTypeById(id: number): BackendFieldType | undefined {
        return fieldTypes.find((ft) => ft.id === id);
    }

    // --- Form Submission ---
    async function handleSubmit(e: Event) {
        e.preventDefault();
        isSubmitting = true;
        errors = {};

        if (!pageData.name) errors["page_name"] = "Nome obrigatório.";
        if (!pageData.path) errors["page_path"] = "Caminho obrigatório.";
        else if (
            !/^\/?[a-z0-9]+(?:[a-z0-9-]*[a-z0-9]+)?(?:\/[a-z0-9]+(?:[a-z0-9-]*[a-z0-9]+)?)*\/?$/.test(
                pageData.path,
            ) &&
            pageData.path !== "/"
        ) {
            errors["page_path"] =
                "Caminho inválido. Use / ou comece com / e use letras minúsculas, números e hífen.";
        }

        if (!pageData.is_group) {
            fields.forEach((field, index) => {
                if (!field.display_name)
                    errors[`field_${index}_display_name`] = "Obrigatório.";
                if (!field.name) errors[`field_${index}_name`] = "Obrigatório.";
                else if (!/^[a-z0-9_]+$/.test(field.name))
                    errors[`field_${index}_name`] = "Inválido.";

                if (field.notification_enabled) {
                    if (
                        !field.notification_days_before ||
                        field.notification_days_before <= 0
                    ) {
                        errors[`field_${index}_notification_days`] =
                            "Dias para notificação deve ser maior que 0.";
                    }
                    if (!field.notification_target_date_part) {
                        errors[`field_${index}_notification_target`] =
                            "Selecione o alvo da notificação (Data Início/Fim).";
                    }
                }
            });
            if (fields.length === 0 && !pageData.is_group)
                errors["fields_general"] =
                    "Pelo menos um campo é necessário para uma página.";
        }

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

        let formattedPath = pageData.path!.trim().toLowerCase();
        if (!formattedPath.startsWith("/")) formattedPath = "/" + formattedPath;
        if (formattedPath.length > 1 && formattedPath.endsWith("/"))
            formattedPath = formattedPath.slice(0, -1);
        if (formattedPath === "") formattedPath = "/";

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
            if (formattedParentPath === "") formattedParentPath = "/";
        }
        if (formattedParentPath === formattedPath) {
            errors["page_path"] =
                "O caminho pai não pode ser igual ao caminho da página.";
            showAlert(
                "Erro de validação no caminho pai.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            isSubmitting = false;
            return;
        }

        const finalData: CreateCustomPageRequest = {
            name: pageData.name!,
            path: formattedPath,
            parent_path: formattedParentPath,
            is_group: pageData.is_group!,
            description: pageData.description || null,
            icon: pageData.icon || null,
            notify_on_new_record: pageData.is_group
                ? false
                : pageData.notify_on_new_record || false, // Added
            requires_acknowledgment: pageData.is_group
                ? false
                : pageData.requires_acknowledgment || false, // Added
            fields: pageData.is_group
                ? []
                : fields.map(({ key, ...f_rest }) => ({
                      // Exclude key from payload
                      ...f_rest,
                      options: f_rest.options ?? null,
                      notification_days_before: f_rest.notification_enabled
                          ? f_rest.notification_days_before
                          : null,
                      notification_target_date_part: f_rest.notification_enabled
                          ? f_rest.notification_target_date_part
                          : null,
                  })),
            permissions: pageData.is_group ? [] : Object.values(permissions),
        };

        try {
            const result = await createCustomPage(finalData);
            if (result.success) {
                showAlert(
                    `${pageData.is_group ? "Grupo" : "Página"} criada com sucesso!`,
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
                        ? "/auto-gerado-grupo"
                        : "/auto-gerado-pagina"}
                    class="input input-bordered w-full bg-base-200"
                    bind:value={pageData.path} 
                    required
                    readonly
                />
                {#if errors.page_path}<span class="text-error text-xs mt-1"
                        >{errors.page_path}</span
                    >{/if}
                {#if errors.page_name_for_path}<span class="text-error text-xs mt-1"
                    >{errors.page_name_for_path}</span
                >{/if}
                <div class="label">
                    <span class="label-text-alt"
                        >Gerado automaticamente a partir do Nome e Grupo Pai. Use / para raiz.</span
                    >
                </div>
            </label>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Grupo Pai (Opcional)</span>
                </div>
                <select
                    class="select select-bordered w-full"
                    bind:value={selectedParentGroupId}
                >
                    <option value={null}>Nenhum (Nível Raiz)</option>
                    {#each availableGroups as group (group.id)}
                        <option value={group.id.toString()}>{group.name} ({group.path})</option>
                    {/each}
                </select>
                <div class="label">
                    <span class="label-text-alt"
                        >Selecione um grupo para aninhar esta página/grupo.</span
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

            <!-- Moved Toggles to the bottom of this fieldset -->
            <div class="form-control md:col-span-1 self-end">
                <label class="label cursor-pointer justify-start gap-2 pt-2">
                    <input
                        type="checkbox"
                        class="toggle toggle-accent"
                        bind:checked={pageData.is_group}
                    />
                    <span class="label-text font-medium"
                        >É um Grupo/Pasta (sem registos)?</span
                    >
                </label>
                <div class="label pt-0">
                    <span class="label-text-alt"
                        >Marque se isto for apenas uma pasta no menu para
                        organizar outras páginas.</span
                    >
                </div>
            </div>

            {#if !pageData.is_group}
                <div class="form-control md:col-span-1 self-end">
                    <label
                        class="label cursor-pointer justify-start gap-2 pt-2"
                    >
                        <input
                            type="checkbox"
                            class="toggle toggle-primary"
                            bind:checked={pageData.notify_on_new_record}
                        />
                        <span class="label-text font-medium"
                            >Notificar em Novos Registos?</span
                        >
                    </label>
                    <div class="label pt-0">
                        <span class="label-text-alt"
                            >Utilizadores com acesso serão notificados.</span
                        >
                    </div>
                </div>

                <div class="form-control md:col-span-1 self-end">
                    <label
                        class="label cursor-pointer justify-start gap-2 pt-2"
                    >
                        <input
                            type="checkbox"
                            class="toggle toggle-secondary"
                            bind:checked={pageData.requires_acknowledgment}
                        />
                        <span class="label-text font-medium"
                            >Exigir Tomar Conhecimento?</span
                        >
                    </label>
                    <div class="label pt-0">
                        <span class="label-text-alt"
                            >Utilizadores terão de confirmar leitura antes de
                            ver detalhes.</span
                        >
                    </div>
                </div>
            {:else}
                <!-- Placeholder for alignment if is_group is true -->
                <div class="md:col-span-1"></div>
                <div class="md:col-span-1"></div>
            {/if}
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
                                <!-- New Header -->
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
                                        <!-- New Cell for can_view_acknowledgments -->
                                        <td class="text-center"
                                            ><input
                                                type="checkbox"
                                                class="checkbox checkbox-xs"
                                                bind:checked={
                                                    () =>
                                                        perm.can_view_acknowledgments ||
                                                        role.is_admin,
                                                    (value) =>
                                                        (perm.can_view_acknowledgments =
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
