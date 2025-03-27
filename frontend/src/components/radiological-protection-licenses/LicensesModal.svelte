<script lang="ts">
  import Table from "@components/common/Table.svelte";
  import FormModal from "@components/common/FormModal.svelte";
  import type { License } from "@lib/types/radiological-protection-licenses";
  import type { TableColumn } from "@lib/types/table";
  import { onMount } from "svelte";

  let loading = $state(true);
  let licenses = $state({});
  let formModalElement = $state<HTMLDialogElement>();

  // Define columns for the table
  const columns: TableColumn[] = [
    { header: "ID", field: "ID" },
    { header: "Âmbito", field: "scope" },
    { header: "Localização", field: "location" },
    { header: "Número da Licença", field: "licenseNumber" },
    { header: "Descrição", field: "description" },
    {
      header: "Data Início",
      field: "dateStartString",
      dateValueField: "dateStart",
    },
    { header: "Data Fim", field: "dateEndString", dateValueField: "dateEnd" },
    {
      header: "Criado em",
      field: "createdAtString",
      dateValueField: "createdAt",
    },
    {
      header: "Atualizado em",
      field: "updatedAtString",
      dateValueField: "updatedAt",
    },
  ];

  let selectedLicenseId: string | null = $state(null);
  let originalLicenseJson: string | null = $state(null);
  let selectedLicense: License | null = $state(null);
  let licenseFiles = $state<
    Record<string, { name: string; path: string; uploadedAt: string }>
  >({});

  // Scope options for the select dropdown
  const scopeOptions = [
    { value: "Medicina Nuclear", label: "Medicina Nuclear" },
    { value: "Radiologia", label: "Radiologia" },
    { value: "Radioterapia", label: "Radioterapia" },
    { value: "Outros", label: "Outros" },
  ];

  // Define form fields for the license
  $effect(() => {
    // This will re-run whenever selectedLicense changes
    licenseFields = [
      {
        id: "scope",
        type: "select",
        label: "Âmbito",
        value: selectedLicense?.scope || "",
        options: scopeOptions,
        required: true,
      },
      {
        id: "location",
        type: "text",
        label: "Localização",
        placeholder: "Introduza a localização",
        value: selectedLicense?.location || "",
        required: true,
        colSpan: 2,
      },
      {
        id: "licenseNumber",
        type: "text",
        label: "Número da Licença",
        placeholder: "Ex: L-12345/2023",
        value: selectedLicense?.licenseNumber || "",
        required: true,
      },
      {
        id: "description",
        type: "textarea",
        label: "Descrição",
        placeholder: "Descrição detalhada da licença",
        value: selectedLicense?.description || "",
        required: true,
        colSpan: 2,
      },
      {
        id: "dateRange",
        type: "dateRange",
        label: "Período de Validade",
        value: selectedLicense
          ? `${selectedLicense.dateStart || ""} - ${selectedLicense.dateEnd || ""}`
          : "",
        required: true,
        colSpan: 2,
      },
    ];
  });

  let licenseFields = [];

  function openModal(id: string, license: License) {
    selectedLicenseId = id;
    // Make a deep copy of the license to avoid modifying the original
    selectedLicense = JSON.parse(JSON.stringify(license));
    originalLicenseJson = JSON.stringify({
      ...selectedLicense,
      files: undefined,
    });

    // Prepare files for the modal
    licenseFiles = selectedLicense.files || {};

    // Open the modal
    console.log("formModalElement", formModalElement);
    formModalElement?.showModal();
  }

  function openNewLicenseModal() {
    selectedLicenseId = null;
    // Initialize with default values for a new license
    selectedLicense = {
      ID: "",
      scope: "Radiologia", // Default option
      location: "",
      licenseNumber: "",
      description: "",
      dateStart: "",
      dateEnd: "",
      dateStartString: "",
      dateEndString: "",
      createdAt: "",
      updatedAt: "",
      createdAtString: "",
      updatedAtString: "",
      files: {},
    };
    licenseFiles = {};
    formModalElement?.showModal();
  }

  async function handleSubmit(formData: Record<string, any>, newFiles: File[]) {
    try {
      const [
        { createLicense, updateLicense },
        { AlertPosition, AlertType, showAlert },
      ] = await Promise.all([
        import("@api/radiological-protection-licenses-api"),
        import("@components/alert/alert"),
      ]);

      // Process date range
      if (formData.dateRange) {
        const [start, end] = formData.dateRange.split(" - ");
        formData.dateStart = start;
        formData.dateEnd = end;
        delete formData.dateRange;
      }

      let success = false;
      let updatedLicense;

      if (selectedLicenseId) {
        // Update existing license
        updatedLicense = await updateLicense(
          selectedLicenseId,
          formData,
          newFiles
        );
        success = !!updatedLicense;

        if (success) {
          showAlert(
            "Licença atualizada com sucesso",
            AlertType.SUCCESS,
            AlertPosition.TOP
          );
          handleUpdated(updatedLicense);
        }
      } else {
        // Create new license
        updatedLicense = await createLicense(formData, newFiles);
        success = !!updatedLicense;

        if (success) {
          showAlert(
            "Licença criada com sucesso",
            AlertType.SUCCESS,
            AlertPosition.TOP
          );
          // Add the new license to the list
          licenses = { ...licenses, [updatedLicense.ID]: updatedLicense };
        }
      }

      return success;
    } catch (error) {
      console.error("Error saving license:", error);
      return false;
    }
  }

  async function handleDelete() {
    if (!selectedLicenseId) return false;

    try {
      const [{ deleteLicense }, { AlertPosition, AlertType, showAlert }] =
        await Promise.all([
          import("@api/radiological-protection-licenses-api"),
          import("@components/alert/alert"),
        ]);

      const success = await deleteLicense(selectedLicenseId);

      if (success) {
        showAlert(
          "Licença eliminada com sucesso",
          AlertType.SUCCESS,
          AlertPosition.TOP
        );
        // Remove the deleted license from the list
        const newLicenses = { ...licenses };
        delete newLicenses[selectedLicenseId];
        licenses = newLicenses;
      }

      return success;
    } catch (error) {
      console.error("Error deleting license:", error);
      return false;
    }
  }

  async function handleFileDelete(licenseId: string, fileId: string) {
    try {
      const [{ deleteFile }, { AlertPosition, AlertType, showAlert }] =
        await Promise.all([
          import("@api/radiological-protection-licenses-api"),
          import("@components/alert/alert"),
        ]);

      const success = await deleteFile(licenseId, fileId);

      if (success) {
        showAlert(
          "Ficheiro eliminado com sucesso",
          AlertType.SUCCESS,
          AlertPosition.TOP
        );
        // Remove the deleted file from the list
        if (licenses[licenseId] && licenses[licenseId].files) {
          const updatedLicenses = { ...licenses };
          delete updatedLicenses[licenseId].files[fileId];
          licenses = updatedLicenses;
        }
      }

      return success;
    } catch (error) {
      console.error("Error deleting file:", error);
      return false;
    }
  }

  function handleUpdated(updatedLicense: License) {
    // Update the license in the list
    const updatedLicenses = { ...licenses };
    updatedLicenses[updatedLicense.ID] = updatedLicense;
    licenses = updatedLicenses;
  }

  onMount(() => {
    (async () => {
      const [{ getLicenses }, { AlertPosition, AlertType, showAlert }] =
        await Promise.all([
          import("@api/radiological-protection-licenses-api"),
          import("@components/alert/alert"),
        ]);

      const licensesOrNull = await getLicenses();
      loading = false;
      if (!licensesOrNull) {
        showAlert(
          "Erro ao carregar licenças",
          AlertType.ERROR,
          AlertPosition.TOP
        );
        return;
      }
      licenses = licensesOrNull;
    })();
  });
</script>

<div class="flex justify-between mb-4">
  <h1 class="text-2xl font-bold">Licenças de Proteção Radiológica</h1>
  <button class="btn btn-primary" on:click={openNewLicenseModal}>
    <i class="fa-solid fa-plus mr-2"></i> Nova Licença
  </button>
</div>

<Table
  data={licenses}
  {columns}
  {loading}
  emptyMessage="Nenhuma licença disponível"
  keyField="ID"
  searchFields={[
    "__searchScope",
    "__searchLocation",
    "__searchLicenseNumber",
    "__searchDescription",
    "dateStartString",
    "dateEndString",
    "createdAtString",
    "updatedAtString",
  ]}
  onRowClick={openModal}
/>

<!-- Form Modal for creating/editing licenses -->
<FormModal
  bind:formModal={formModalElement}
  title={selectedLicenseId ? "Editar Licença" : "Nova Licença"}
  fields={licenseFields}
  recordId={selectedLicenseId}
  showFiles={true}
  files={licenseFiles}
  onSubmit={handleSubmit}
  onDelete={handleDelete}
  onFileDeleted={handleFileDelete}
  showDeleteButton={!!selectedLicenseId}
  deleteButtonText="Eliminar Licença"
  submitButtonText={selectedLicenseId ? "Atualizar Licença" : "Criar Licença"}
  fileUploadLabel="Documentos da Licença"
  fileUploadHint="Adicione documentação da licença (PDF, DOCX, etc.)"
/>
