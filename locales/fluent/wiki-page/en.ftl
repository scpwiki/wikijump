### Wiki Page View

wiki-page-category = category: { $category }

wiki-page-revision = revision: { $revision }

wiki-page-last-edit = last edited: { $date } ({ $days ->
  [0] today
  [1] yesterday
  *[other] { $days } days ago
})

wiki-page-source = Page source

wiki-page-view-source = View Source

wiki-page-revision-history = Page revision history

wiki-page-revision-number = Revision #

wiki-page-revision-created-at = Edit time

wiki-page-revision-user = User

wiki-page-revision-comments = Comments

wiki-page-revision-rollback = Revert

wiki-page-revision-type = Type
  .create = Create
  .regular = Edit
  .move = Move
  .delete = Delete
  .rollback = Rollback
  .undelete = Restore
  .undo = Undo

### Wiki Page Vote

wiki-page-vote = Page rating
  .set = Cast vote
  .remove = Cancel vote
  .list = List votes
  .score = Rating
  .toast-set = Successfully voted.
  .toast-remove = Successfully removed vote.

### Wiki Page Edit

wiki-page-edit = Edit the page
  .toast = Page saved.

wiki-page-create = Create new page

wiki-page-move = Move page
  .new-slug = New slug

wiki-page-layout = Page layout
  .default = Default layout
  .wikidot = Wikidot (Legacy)
  .wikijump = Wikijump

wiki-page-delete = Delete page

wiki-page-restore = Restore page
  .select = Select page to restore

wiki-page-deleted = Deleted at { $datetime }

### Wiki Page Files

wiki-page-file-no-files = No files for this page.

wiki-page-file-upload =
  .select = Select file:
  .name = File name:

wiki-page-file-move-destination-page = Destination page

wiki-page-file = Page files
  .name = File name
  .created-at = Created at
  .updated-at = Updated at
  .mime = File type
  .size = File size
  .page = Page

wiki-page-file-revision-type = Type
  .create = Create
  .regular = Edit
  .move = Move
  .delete = Delete
  .rollback = Revert
  .undelete = Restore
  .undo = Undo

wiki-page-file-restore = Restore
  .new-page = Destination page
  .new-name = New file name

### Wiki page lock

wiki-page-lock = Lock Page
  .permission-only = Permission Only
  .author-or-permission-only = Permission/Author Only
  .permission-only-text = Only members with lock bypass permission can edit.
  .author-or-permission-only-text = Members with permission and page authors can edit.
  .reason = Reason (optional)
  .expires-at = (Optional) Set a date when this lock will automatically expire:
  .override = Override existing lock
  .history = Lock History
  .history-type = Type
  .history-user = Locked by
  .history-reason = Lock reason
  .history-created = Created at
  .history-expires = Expires at
  .history-status = Status
  .history-active = Active
  .history-expired = Expired
  .history-removed = Removed
  .history-overridden = Overridden
  .history-none = No lock history for this page.
  .remove = Remove

### Wiki page parents

wiki-page-parent = Page parents

### Blueprint page fallback strings

wiki-page-missing = The page //{ $slug }// you want to access does not exist.

    { " *" } [/{ $slug }/edit create this page].

wiki-page-private = + Private content

    This area of the website is private and you don't have access to it. If you believe you need access to this area please contact the web site administrators.

wiki-page-banned = + You have been banned

    You are currently banned from this site, and the site settings do not allow banned users to view pages.

wiki-page-no-render = Content not shown.
