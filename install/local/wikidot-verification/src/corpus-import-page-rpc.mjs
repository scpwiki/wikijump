export async function getCorpusImportPage(args, rpc, slug) {
  return await rpc(args, 'page_get', {
    site_id: args.siteId,
    page: slug,
    details: { wikitext: false, compiled: false },
  });
}

export async function getCorpusImportFile(args, rpc, pageId, filename) {
  return await rpc(args, 'file_get', {
    site_id: args.siteId,
    page_id: pageId,
    file: filename,
    details: { data: false },
  }, { siteId: args.siteId, pageRef: pageId });
}

export async function createCorpusImportPage(args, rpc, row, source) {
  return await rpc(args, 'page_create', {
    site_id: args.siteId,
    wikitext: source,
    title: row.title || row.title_shown || row.fullname,
    alt_title: null,
    slug: row.fullname,
    layout: 'wikidot',
    revision_comments: 'local scp-wiki mirror import from scp-wiki-translation corpus',
    user_id: args.userId,
    bypass_filter: true,
    ip_address: args.ipAddress,
  });
}

export async function rerenderCorpusImportPage(args, rpc, pageId, categoryId) {
  return await rpc(args, 'page_rerender', {
    site_id: args.siteId,
    category_id: categoryId,
    page_id: pageId,
  });
}

export function corpusImportCategoryName(slug) {
  const index = slug.lastIndexOf(':');
  return index === -1 ? '_default' : slug.slice(0, index);
}
