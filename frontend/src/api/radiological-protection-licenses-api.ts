import { handleFetch } from "@api/fetch-handler";
import API_BASE_URL from "@api/base-url";

import type {
  License,
  Licenses,
} from "@lib/types/radiological-protection-licenses";
import { DMYHMSToDate, DMYToDate } from "@utils/date-utils";
import { Locations } from "@lib/types/locations";
import { toSearchString } from "@utils/search-utils";

export async function getLicenses(): Promise<Licenses | null> {
  const response = await handleFetch(
    `${API_BASE_URL}/ers/radiological-protection/licenses`,
    {
      method: "GET",
      credentials: "include",
    },
  );

  if (response.ok) {
    const json = await response.json();
    const entries = Object.values(json) as License[];
    for (let i = 0, len = entries.length; i < len; i++) {
      const entry = entries[i];
      entry.locationValue = entry.location as unknown as number;

      entry.dateStart = DMYToDate(entry.dateStartString);
      entry.dateEnd = DMYToDate(entry.dateEndString);
      entry.createdAt = DMYHMSToDate(entry.createdAtString);
      entry.updatedAt = DMYHMSToDate(entry.updatedAtString);

      entry.location = Locations[entry.location as unknown as number];

      entry.__searchScope = toSearchString(entry.scope);
      entry.__searchLocation = toSearchString(entry.location as string);
      entry.__searchLicenseNumber = entry.licenseNumber.toString();

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
  return null;
}

/**
 * @returns boolean indicating if the request was successful, the id of the contract created and the id of the first file uploaded
 */
export async function uploadLicense(
  formData: FormData,
): Promise<[boolean, number, number]> {
  const response = await handleFetch(
    `${API_BASE_URL}/ers/radiological-protection/licenses`,
    {
      method: "POST",
      body: formData,
      credentials: "include",
    },
  );
  const [licenseId, fileId] = (await response.text()).split(",");

  return [response.ok, parseInt(licenseId, 10), parseInt(fileId, 10)];
}

export async function uploadLicenseFiles(
  licenseId: string,
  files: File[],
): Promise<[boolean, number]> {
  const formData = new FormData();
  for (let i = 0, len = files.length; i < len; i++) {
    const file = files[i];
    formData.append("files", file, `${file.name}_${file.size}`);
  }

  const resp = await handleFetch(
    `${API_BASE_URL}/ers/radiological-protection/licenses/${licenseId}/files`,
    {
      method: "POST",
      credentials: "include",
      body: formData,
    },
  );

  if (!resp.ok) {
    return [false, -1];
  }

  return [resp.ok, parseInt(await resp.text(), 10)];
}

export async function deleteLicenseFile(
  licenseId: string,
  fileId: string,
): Promise<boolean> {
  const resp = await handleFetch(
    `${API_BASE_URL}/ers/radiological-protection/licenses/${licenseId}/files/${fileId}`,
    {
      method: "DELETE",
      credentials: "include",
    },
  );

  return resp.ok;
}

export async function deleteLicense(licenseId: string): Promise<boolean> {
  const resp = await handleFetch(
    `${API_BASE_URL}/ers/radiological-protection/licenses/${licenseId}`,
    {
      method: "DELETE",
      credentials: "include",
    },
  );

  return resp.ok;
}

export async function updateLicense(
  licenseId: string,
  license: License,
): Promise<boolean> {
  const resp = await handleFetch(
    `${API_BASE_URL}/ers/radiological-protection/licenses/${licenseId}`,
    {
      method: "PUT",
      credentials: "include",
      body: JSON.stringify({
        scope: license.scope,
        license_number: license.licenseNumber,
        start_date: license.dateStartString,
        end_date: license.dateEndString,
        location: license.locationValue,
        description: license.description,
      }),
      headers: {
        "Content-Type": "application/json",
      },
    },
  );
  return resp.ok;
}
