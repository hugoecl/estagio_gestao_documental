export const Locations = [
  "Viana do Castelo",
  "Braga",
  "Porto",
  "Vila Real",
] as const;

export const LocationsObject = Locations.map((location, index) => ({
  value: index,
  label: location,
}));
