export class UsageError extends Error {}

export function readRequiredOptionValue(argv, index, flag) {
  const value = argv[index + 1];
  if (value === undefined) {
    throw new UsageError(`${flag} needs a value`);
  }
  return value;
}

export function parsePositiveIntegerOption(value, flag) {
  if (!/^[0-9]+$/u.test(value)) {
    throw new UsageError(`${flag} must be a positive integer`);
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new UsageError(`${flag} must be a positive integer`);
  }
  return parsed;
}
