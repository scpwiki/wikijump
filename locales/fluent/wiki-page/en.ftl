### Wiki Page View

wiki-page-category = category: { $category }

wiki-page-revision = revision: { $revision }

wiki-page-last-edit = last edited: { $date } ({ $days ->
  [0] today
  [1] yesterday
  *[other] { $days } days ago
})

wiki-page-view-source = View Source

wiki-page-revision-number = Revision #

wiki-page-revision-created-at = Creation

wiki-page-revision-user = User

wiki-page-revision-comments = Comments

wiki-page-revision-rollback = Revert

### Wiki Page Vote

wiki-page-vote-set = Cast vote

wiki-page-vote-remove = Cancel vote

wiki-page-vote-list = List votes

wiki-page-vote-score = Rating

### Wiki Page Edit

wiki-page-move-new-slug = New slug

wiki-page-layout-default = Default layout

wiki-page-layout-wikidot = Wikidot (Legacy)

wiki-page-layout-wikijump = Wikijump

### Special Page fallback strings

wiki-page-missing = The page //{ $slug }// you want to access does not exist.

    { " *" } [/{ $slug }/edit create this page].

wiki-page-private = + Private content

    This area of the website is private and you don't have access to it. If you believe you need access to this area please contact the web site administrators.

wiki-page-banned = + You have been banned

    You are currently banned from this site, and the site settings do not allow banned users to view pages.

wiki-page-site-slug = <h1>No { -service-name } site exists with this address.</h1>
    <p>
      <a href="https://{ $slug }.{ $domain }/">{ $slug }.{ $domain }</a> does not exist.
      Return to <a href="https://{ $domain }/">{ -service-name }</a>.
    </p>

wiki-page-site-custom = <h1>No { -service-name } site exists with this address.</h1>
    <p>
      No site has the custom domain <a href="https://{ $custom_domain }/">{ $custom_domain }</a>.
      Return to <a href="https://{ $domain }/">{ -service-name }</a>.
    </p>

wiki-page-no-render = Content not shown.
