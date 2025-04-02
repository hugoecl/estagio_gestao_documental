import type { Locations } from "@lib/types/locations";
import type { FileData } from "@lib/types/files";

export interface License {
  scope: string;
  licenseNumber: number;
  dateStart: Date;
  dateEnd: Date;
  description?: string;

  location: (typeof Locations)[number];
  locationValue: number;

  createdAt: Date;
  updatedAt: Date;

  files: Record<number, FileData>;

  // Things made for the sake of performance primarily when searching that are added when the data is fetched

  dateStartString: string;
  dateEndString: string;
  createdAtString: string;
  updatedAtString: string;

  __searchScope: string;
  __searchLocation: string;
  __searchLicenseNumber: string;
  __searchDescription?: string;
}

export type Licenses = Record<number, License>;
