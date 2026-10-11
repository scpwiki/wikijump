export function canReuseExistingPageForDbImport(args, { replaceExistingRevision = false } = {}) {
  return Boolean(args.adoptExisting || args.replaceExisting || replaceExistingRevision);
}

export function initialImportItemState(args) {
  return args.createMode === 'db' && args.skipRerender ? 'shell_ready' : 'render_pending';
}
