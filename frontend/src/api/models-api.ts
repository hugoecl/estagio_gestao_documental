import API_BASE_URL from "@api/base-url";
import type { Model, Models } from "@lib/types/model";
import { toSearchString } from "@utils/search-utils";
import { handleFetch } from "./fetch-handler";

export async function uploadModel(
  formData: FormData,
): Promise<[boolean, number, number]> {
  const resp = await handleFetch(`${API_BASE_URL}/quality/models`, {
    method: "POST",
    body: formData,
    credentials: "include",
  });

  const [modelId, fileId] = (await resp.text()).split(",");
  return [resp.ok, parseInt(modelId), parseInt(fileId)];
}

export async function getModels(): Promise<Models | null> {
  const resp = await handleFetch(`${API_BASE_URL}/quality/models`, {
    credentials: "include",
  });

  if (!resp.ok) {
    return null;
  }

  const json = await resp.json();
  const entries = Object.values(json) as Model[];
  for (let i = 0, len = entries.length; i < len; i++) {
    const entry = entries[i];

    entry.__searchName = toSearchString(entry.name);
    if (entry.description) {
      entry.__searchDescription = toSearchString(entry.description);
    }

    const files = entry.files;
    for (const key in files) {
      files[key].name = files[key].path.split("/").at(-1)!;
    }
  }
  return json;
}
