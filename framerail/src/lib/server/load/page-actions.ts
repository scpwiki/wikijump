import {
  layoutAction,
  pageDeleteAction,
  pageEditAction,
  pageEditPermissionAction,
  pageMoveAction
} from "$lib/server/load/page-edit-actions"
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
  pageParentGetAction,
  pageParentSetAction,
  pageScoreAction,
  pageVoteCancelAction,
  pageVoteCastAction,
  pageVoteGetAction
} from "$lib/server/load/page-relation-actions"
import {
  pageDeletedGetAction,
  pageHistoryAction,
  pageRestoreAction,
  pageRevisionAction,
  pageRollbackAction
} from "$lib/server/load/page-revision-actions"

export const pageActions = {
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
