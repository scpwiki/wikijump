import {
  layoutAction,
  loadPage,
  pageDeleteAction,
  pageDeletedGetAction,
  pageEditAction,
  pageEditPermissionAction,
  pageFileDeleteAction,
  pageFileEditAction,
  pageFileHistoryAction,
  pageFileListAction,
  pageFileMoveAction,
  pageFileRestoreAction,
  pageFileRollbackAction,
  pageFileUploadAction,
  pageHistoryAction,
  pageMoveAction,
  pageParentGetAction,
  pageParentSetAction,
  pageRestoreAction,
  pageRevisionAction,
  pageRollbackAction,
  pageScoreAction,
  pageVoteCancelAction,
  pageVoteCastAction,
  pageVoteGetAction
} from "$lib/server/load/page"

export async function load({ params, request, cookies, parent }) {
  return loadPage(params.slug, params.extra, request, cookies, parent)
}

export const actions = {
  delete: pageDeleteAction,
  edit: pageEditAction,
  editPermission: pageEditPermissionAction,
  fileList: pageFileListAction,
  fileUpload: pageFileUploadAction,
  fileDelete: pageFileDeleteAction,
  fileEdit: pageFileEditAction,
  fileMove: pageFileMoveAction,
  fileRestore: pageFileRestoreAction,
  fileHistory: pageFileHistoryAction,
  fileRollback: pageFileRollbackAction,
  history: pageHistoryAction,
  revision: pageRevisionAction,
  rollback: pageRollbackAction,
  layout: layoutAction,
  move: pageMoveAction,
  parentSet: pageParentSetAction,
  parentGet: pageParentGetAction,
  voteGet: pageVoteGetAction,
  voteCast: pageVoteCastAction,
  voteCancel: pageVoteCancelAction,
  score: pageScoreAction,
  deletedGet: pageDeletedGetAction,
  restore: pageRestoreAction
}
