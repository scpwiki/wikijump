import {
  pageDeleteAction,
  pageEditAction,
  pageEditPermissionAction,
  pageLayoutAction,
  pageMoveAction
} from "$lib/server/load/page/page-edit-actions"
import {
  pageFileDeleteAction,
  pageFileEditAction,
  pageFileHistoryAction,
  pageFileListAction,
  pageFileMoveAction,
  pageFileRestoreAction,
  pageFileRollbackAction,
  pageFileUploadAction
} from "$lib/server/load/page/page-file-actions"
import {
  pageParentGetAction,
  pageParentSetAction,
  pageScoreAction,
  pageVoteCastAction,
  pageVoteListAction,
  pageVoteRemoveAction
} from "$lib/server/load/page/page-relation-actions"
import {
  pageDeletedGetAction,
  pageHistoryAction,
  pageRestoreAction,
  pageRevisionAction,
  pageRollbackAction
} from "$lib/server/load/page/page-revision-actions"

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
  layout: pageLayoutAction,
  move: pageMoveAction,
  parentSet: pageParentSetAction,
  parentGet: pageParentGetAction,
  voteGet: pageVoteListAction,
  voteCast: pageVoteCastAction,
  voteCancel: pageVoteRemoveAction,
  score: pageScoreAction,
  deletedGet: pageDeletedGetAction,
  restore: pageRestoreAction
}
