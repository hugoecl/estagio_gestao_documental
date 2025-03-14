const VALID_FIRST = 0b0010_1110_1110; // 1,2,3,5,6,8
const VALID_7X = 0b0010_1010_111; // 70,71,72,74,75,77,79
const VALID_9X = 0b0011_0000_11; // 90,91,98,99

export function validateNIF(nif: string): boolean {
  if (nif.length !== 9) return false;

  const c0 = nif.charCodeAt(0);
  const c1 = nif.charCodeAt(1);

  if (c0 < 48 || c0 > 57 || c1 < 48 || c1 > 57) return false;

  // Optimize branching with lookup masks for first digit
  const d0 = c0 - 48;
  const isFirstDigitValid = ((1 << d0) & VALID_FIRST) !== 0;

  // If first digit isn't valid on its own, check two-digit patterns
  if (!isFirstDigitValid) {
    const d1 = c1 - 48;
    let valid = false;

    switch (d0) {
      case 4:
        valid = d1 === 5;
        break; // Only 45 is valid
      case 7:
        valid = ((1 << d1) & VALID_7X) !== 0;
        break; // 7 with 0,1,2,4,5,7,9
      case 9:
        valid = ((1 << d1) & VALID_9X) !== 0;
        break; // 9 with 0,1,8,9
      default:
        valid = false;
    }

    if (!valid) return false;
  }

  const c2 = nif.charCodeAt(2);
  const c3 = nif.charCodeAt(3);
  const c4 = nif.charCodeAt(4);
  const c5 = nif.charCodeAt(5);
  const c6 = nif.charCodeAt(6);
  const c7 = nif.charCodeAt(7);
  const c8 = nif.charCodeAt(8);

  if (
    c2 < 48 ||
    c2 > 57 ||
    c3 < 48 ||
    c3 > 57 ||
    c4 < 48 ||
    c4 > 57 ||
    c5 < 48 ||
    c5 > 57 ||
    c6 < 48 ||
    c6 > 57 ||
    c7 < 48 ||
    c7 > 57 ||
    c8 < 48 ||
    c8 > 57
  ) {
    return false;
  }

  const sum =
    (c0 - 48) * 9 +
    ((c1 - 48) << 3) + // x8 (bitwise shift)
    (c2 - 48) * 7 +
    (c3 - 48) * 6 +
    (c4 - 48) * 5 +
    (c5 - 48) * 4 +
    (c6 - 48) * 3 +
    ((c7 - 48) << 1); // x2 (bitwise shift)

  const remainder = sum % 11;
  return (remainder < 2 ? 0 : 11 - remainder) === c8 - 48;
}
