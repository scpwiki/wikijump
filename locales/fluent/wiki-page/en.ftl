### Wiki Page View

wiki-page-category = category: { $category }

wiki-page-revision = revision: { $revision }

wiki-page-last-edit = last-edited: { $date } ({ $days ->
  [0] today
  [1] yesterday
  *[other] { $days } days ago
})

### Special Page fallback strings

wiki-page-missing = The page //{ $slug }// you want to access does not exist.

    { " *" } [/{ $slug }/edit create this page].

wiki-page-private = + Private content

    This area of the website is private and you don't have access to it. If you believe you need access to this area please contact the web site administrators.

wiki-page-site = + No { -service-name } site exists with this address.

    "@@{ $slug }.{ $domain }@@" does not exist. You can visit the main { -service-name } site here: { $domain }
