import {
  pageFileDeleteAction,
  pageFileEditAction,
  pageFileHistoryAction,
  pageFileListAction,
  pageFileMoveAction,
  pageFileRestoreAction,
  pageFileRollbackAction,
  pageFileUploadAction
} from "$lib/server/load/page-file-actions"
import {
  layoutAction,
  loadPage,
  pageDeleteAction,
  pageEditAction,
  pageEditPermissionAction,
  pageMoveAction,
  pageParentGetAction,
  pageParentSetAction,
  pageScoreAction,
  pageVoteCancelAction,
  pageVoteCastAction,
  pageVoteGetAction
} from "$lib/server/load/page"
import {
  pageDeletedGetAction,
  pageHistoryAction,
  pageRestoreAction,
  pageRevisionAction,
  pageRollbackAction
} from "$lib/server/load/page-revision-actions"

export async function load({ params, request, cookies, locals }) {
  return loadPage(params.slug, params.extra, request, cookies, locals)
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
