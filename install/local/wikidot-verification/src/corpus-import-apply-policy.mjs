export function canReuseExistingPageForDbImport(args, { replaceExistingRevision = false } = {}) {
  return Boolean(args.adoptExisting || args.replaceExisting || replaceExistingRevision);
}
