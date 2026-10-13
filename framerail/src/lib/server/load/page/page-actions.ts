import {
  pageDeleteAction,
  pageDeletePermissionAction,
  pageEditAction,
  pageEditPermissionAction,
  pageLayoutAction,
  pageMoveAction,
  pageMovePermissionAction
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
  pageBacklinksAction,
  pageParentGetAction,
  pageParentSetAction,
  pageScoreAction,
  pageWatchersAction,
  pageVoteCastAction,
  pageVoteListAction,
  pageVoteRemoveAction,
  wikidotLegacyRateAction,
  wikidotLegacySetTagsAction
} from "$lib/server/load/page/page-relation-actions"
import {
  pageDeletedGetAction,
  pageHistoryAction,
  pageRestoreAction,
  pageRevisionAction,
  pageRevisionDiffAction,
  pageRollbackAction
} from "$lib/server/load/page/page-revision-actions"
import {
  pageLockCreateAction,
  pageLockHistoryAction,
  pageLockRemoveAction
} from "$lib/server/load/page/page-lock-actions"
import {
  membershipApplicationAction,
  membershipEmailInvitationAction,
  membershipJoinAction,
  membershipPasswordAction
} from "$lib/server/load/page/page-membership-actions"

export const pageActions = {
  backlinks: pageBacklinksAction,
  delete: pageDeleteAction,
  edit: pageEditAction,
  editPermission: pageEditPermissionAction,
  movePermission: pageMovePermissionAction,
  deletePermission: pageDeletePermissionAction,
  fileList: pageFileListAction,
  fileUpload: pageFileUploadAction,
  fileDelete: pageFileDeleteAction,
  fileEdit: pageFileEditAction,
  fileMove: pageFileMoveAction,
  fileRestore: pageFileRestoreAction,
  fileHistory: pageFileHistoryAction,
  fileRollback: pageFileRollbackAction,
  history: pageHistoryAction,
  lockCreate: pageLockCreateAction,
  lockHistory: pageLockHistoryAction,
  lockRemove: pageLockRemoveAction,
  revision: pageRevisionAction,
  revisionDiff: pageRevisionDiffAction,
  rollback: pageRollbackAction,
  layout: pageLayoutAction,
  move: pageMoveAction,
  parentSet: pageParentSetAction,
  parentGet: pageParentGetAction,
  voteGet: pageVoteListAction,
  voteCast: pageVoteCastAction,
  voteCancel: pageVoteRemoveAction,
  score: pageScoreAction,
  watchers: pageWatchersAction,
  legacyRate: wikidotLegacyRateAction,
  legacySetTags: wikidotLegacySetTagsAction,
  membershipApplication: membershipApplicationAction,
  membershipEmailInvitation: membershipEmailInvitationAction,
  membershipJoin: membershipJoinAction,
  membershipPassword: membershipPasswordAction,
  deletedGet: pageDeletedGetAction,
  restore: pageRestoreAction
}
