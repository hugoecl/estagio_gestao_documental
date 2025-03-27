<script lang="ts">
  import FormModal from "@components/common/FormModal.svelte";
  import Table from "@components/common/Table.svelte";
  import type { License } from "@lib/types/radiological-protection-licenses";
  import type { TableColumn } from "@lib/types/table";
  import { currentModal } from "@stores/modal-store";
  import { onMount } from "svelte";

  let loading = $state(true);
  let licenses = $state({});
  let modal: HTMLDialogElement;

  const columns: TableColumn[] = [
    { header: "ID", field: "ID" },
    { header: "Âmbito", field: "scope" },
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
  let selecetdLicense: License | null = $state(null);

  const fields: FormField[] = $derived([
    {
      id: "scope",
      type: "text",
      label: "Âmbito",
      placeholder: "Âmbito",
      value: selecetdLicense ? (selecetdLicense as License).scope : "",
    },
    {
      id: "licenseNumber",
      type: "number",
      label: "Número da Licença",
      placeholder: "Número da Licença",
      value: selecetdLicense ? (selecetdLicense as License).licenseNumber : "",
    },
    {
      id: "dateStartString",
      type: "date",
      label: "Data Início",
      value: selecetdLicense
        ? (selecetdLicense as License).dateStartString
        : "",
    },
    {
      id: "dateEndString",
      type: "date",
      label: "Data Fim",
      value: selecetdLicense ? (selecetdLicense as License).dateEndString : "",
    },
    {
      id: "description",
      type: "textarea",
      label: "Descrição (Opcional)",
      placeholder: "Descrição",
      required: false,
      value: selecetdLicense ? (selecetdLicense as License).description : "",
      colSpan: 2,
    },
  ]);

  function openModal(id: string, license: License) {
    selectedLicenseId = id;
    selecetdLicense = $state.snapshot(license);
    originalLicenseJson = JSON.stringify({
      ...selecetdLicense,
      files: undefined,
    });
    modal.show();
    // get the first child of the modal
    const modalBox = modal.children[0] as HTMLDivElement;
    currentModal.set(modalBox);
  }

  async function handleSubmit(
    formData: Record<string, any>,
    files: File[]
  ): Promise<boolean> {
    // @ts-ignore javascript can take string as indexes
    licenses[selectedLicenseId!] = {
      ...selecetdLicense!,
      ...formData,
    };
    return true;
  }

  async function handleDeleted(): Promise<boolean> {
    // @ts-ignore javascript can take string as indexes
    delete licenses[selectedLicenseId];
    return true;
  }

  async function handleFileDeleted(
    licenseId: string,
    fileId: string
  ): Promise<boolean> {
    // @ts-ignore javascript can take string as indexes
    delete licenses[licenseId].files[fileId];
    return true;
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

<FormModal
  bind:formModal={modal}
  title={selectedLicenseId ? "Editar Licença" : "Nova Licença"}
  {fields}
  recordId={selectedLicenseId || ""}
  showFiles={true}
  files={selecetdLicense ? (selecetdLicense as License).files : {}}
  onSubmit={handleSubmit}
  onDelete={handleDeleted}
  onFileDeleted={handleFileDeleted}
  showDeleteButton={!!selectedLicenseId}
  submitButtonText={selectedLicenseId ? "Atualizar" : "Criar"}
/>

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
