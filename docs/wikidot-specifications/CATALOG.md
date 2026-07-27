# Wikidot feature catalog

This is the human-readable index of every feature extracted from the frozen local Wikidot documentation corpus. The authoritative machine-readable form is [catalog.json](catalog.json); source-page disposition is recorded in [source-coverage.json](source-coverage.json).

## Summary

- Features: 210
- Corpus pages enumerated: 1806
- Corpus pages connected to one or more feature IDs: 1801
- Corpus pages classified without a feature ID: 5
- Unclassified corpus pages: 0

Features by category:

- `api`: 15
- `data-forms`: 27
- `layout`: 3
- `module`: 74
- `platform`: 42
- `site-structure`: 13
- `wiki-syntax`: 36

## Status meanings

- `documented`: the snapshot contains a direct behavioral reference.
- `documented-deprecated`: the behavior is documented but explicitly deprecated.
- `documented-negative`: the documented behavior is that an interface is absent or removed.
- `documented-plan-capability`: the behavior is tied to a documented account/site plan.
- `high-level-documentation`: the feature is stated, but implementation details require live-oracle work.
- `partially-documented`: the canonical page is empty or incomplete.
- `invocation-only`: the corpus proves a module name and use site but has no dedicated contract.

## Features

| Feature ID | Title | Documentation status | Specification |
|---|---|---|---|
| `api-categories-select` | Wikidot API: categories.select | `documented` | [specification](specifications/api/api-categories-select.md) |
| `api-deleted-methods` | Removed Wikidot API methods | `documented-negative` | [specification](specifications/api/api-deleted-methods.md) |
| `api-files-get-meta` | Wikidot API: files.get_meta | `documented` | [specification](specifications/api/api-files-get-meta.md) |
| `api-files-get-one` | Wikidot API: files.get_one | `documented` | [specification](specifications/api/api-files-get-one.md) |
| `api-files-save-one` | Wikidot API: files.save_one | `documented` | [specification](specifications/api/api-files-save-one.md) |
| `api-files-select` | Wikidot API: files.select | `documented` | [specification](specifications/api/api-files-select.md) |
| `api-overview` | Wikidot API overview | `documented` | [specification](specifications/api/api-overview.md) |
| `api-pages-get-meta` | Wikidot API: pages.get_meta | `documented` | [specification](specifications/api/api-pages-get-meta.md) |
| `api-pages-get-one` | Wikidot API: pages.get_one | `documented` | [specification](specifications/api/api-pages-get-one.md) |
| `api-pages-save-one` | Wikidot API: pages.save_one | `documented` | [specification](specifications/api/api-pages-save-one.md) |
| `api-pages-select` | Wikidot API: pages.select | `documented` | [specification](specifications/api/api-pages-select.md) |
| `api-posts-get` | Wikidot API: posts.get | `documented` | [specification](specifications/api/api-posts-get.md) |
| `api-posts-select` | Wikidot API: posts.select | `documented` | [specification](specifications/api/api-posts-select.md) |
| `api-tags-select` | Wikidot API: tags.select | `documented` | [specification](specifications/api/api-tags-select.md) |
| `api-users-get-me` | Wikidot API: users.get_me | `documented` | [specification](specifications/api/api-users-get-me.md) |
| `data-forms-checkbox-field` | The 'checkbox' field type | `documented` | [specification](specifications/data-forms/data-forms-checkbox-field.md) |
| `data-forms-creating-new-page` | Creating a new page | `documented` | [specification](specifications/data-forms/data-forms-creating-new-page.md) |
| `data-forms-css-styling` | CSS Styling | `documented` | [specification](specifications/data-forms/data-forms-css-styling.md) |
| `data-forms-dataforms-and-listpages` | Using the data in ListPages modules | `documented` | [specification](specifications/data-forms/data-forms-dataforms-and-listpages.md) |
| `data-forms-date-field` | The 'date' field type | `documented` | [specification](specifications/data-forms/data-forms-date-field.md) |
| `data-forms-deleting-form` | Deleting a form | `documented` | [specification](specifications/data-forms/data-forms-deleting-form.md) |
| `data-forms-displaying` | Displaying the results | `documented` | [specification](specifications/data-forms/data-forms-displaying.md) |
| `data-forms-field-properties` | Field Properties | `documented` | [specification](specifications/data-forms/data-forms-field-properties.md) |
| `data-forms-file-field` | The 'file' field type | `documented` | [specification](specifications/data-forms/data-forms-file-field.md) |
| `data-forms-hidden-field` | The 'hidden' field type | `documented` | [specification](specifications/data-forms/data-forms-hidden-field.md) |
| `data-forms-hints` | Hints & Tips | `documented` | [specification](specifications/data-forms/data-forms-hints.md) |
| `data-forms-howto` | How to create a new data form | `documented` | [specification](specifications/data-forms/data-forms-howto.md) |
| `data-forms-images` | Images in Data Forms | `documented` | [specification](specifications/data-forms/data-forms-images.md) |
| `data-forms-links` | Links | `documented` | [specification](specifications/data-forms/data-forms-links.md) |
| `data-forms-output-style` | Styling the output of a field | `documented` | [specification](specifications/data-forms/data-forms-output-style.md) |
| `data-forms-overview` | Data Forms | `documented` | [specification](specifications/data-forms/data-forms-overview.md) |
| `data-forms-pagepath` | The Pagepath concept | `documented` | [specification](specifications/data-forms/data-forms-pagepath.md) |
| `data-forms-pagepath-field` | The 'pagepath' field type | `documented` | [specification](specifications/data-forms/data-forms-pagepath-field.md) |
| `data-forms-password-field` | The 'password' field type | `documented` | [specification](specifications/data-forms/data-forms-password-field.md) |
| `data-forms-select-field` | The 'select' field type | `documented` | [specification](specifications/data-forms/data-forms-select-field.md) |
| `data-forms-selecting-and-sorting` | Selecting & Sorting by Data Form fields | `documented` | [specification](specifications/data-forms/data-forms-selecting-and-sorting.md) |
| `data-forms-static-field` | The 'static' field type | `documented` | [specification](specifications/data-forms/data-forms-static-field.md) |
| `data-forms-tags` | Tags | `documented` | [specification](specifications/data-forms/data-forms-tags.md) |
| `data-forms-text-field` | The 'text' field type | `documented` | [specification](specifications/data-forms/data-forms-text-field.md) |
| `data-forms-url-field` | The 'url' field type | `documented` | [specification](specifications/data-forms/data-forms-url-field.md) |
| `data-forms-wiki-field` | The 'wiki' field type | `documented` | [specification](specifications/data-forms/data-forms-wiki-field.md) |
| `data-forms-youtube` | YouTube and other external content | `documented` | [specification](specifications/data-forms/data-forms-youtube.md) |
| `layout-custom` | Custom page layouts | `documented` | [specification](specifications/layout/layout-custom.md) |
| `layout-forum` | Forum layout structure | `documented` | [specification](specifications/layout/layout-forum.md) |
| `layout-page` | Default page layout | `documented` | [specification](specifications/layout/layout-page.md) |
| `module-ad` | Ad Module | `invocation-only` | [specification](specifications/module/module-ad.md) |
| `module-admoduleabovecontent` | Ad Module Above Content Module | `invocation-only` | [specification](specifications/module/module-admoduleabovecontent.md) |
| `module-admoduleabovesidebar` | Ad Module Above Sidebar Module | `invocation-only` | [specification](specifications/module/module-admoduleabovesidebar.md) |
| `module-admodulebelowcontent` | Ad Module Below Content Module | `invocation-only` | [specification](specifications/module/module-admodulebelowcontent.md) |
| `module-admodulebelowfooter` | Ad Module Below Footer Module | `invocation-only` | [specification](specifications/module/module-admodulebelowfooter.md) |
| `module-admodulebelowsidebar` | Ad Module Below Sidebar Module | `invocation-only` | [specification](specifications/module/module-admodulebelowsidebar.md) |
| `module-adsenseunit` | AdSenseUnit Module | `documented` | [specification](specifications/module/module-adsenseunit.md) |
| `module-anonymousnotificationsunsubscribe` | Anonymous Notifications Unsubscribe Module | `invocation-only` | [specification](specifications/module/module-anonymousnotificationsunsubscribe.md) |
| `module-backlinks` | Backlinks Module | `documented` | [specification](specifications/module/module-backlinks.md) |
| `module-categories` | Categories Module | `documented` | [specification](specifications/module/module-categories.md) |
| `module-childpages` | ChildPages Module | `documented` | [specification](specifications/module/module-childpages.md) |
| `module-clone` | Clone Module | `documented` | [specification](specifications/module/module-clone.md) |
| `module-comments` | Comments Module | `documented` | [specification](specifications/module/module-comments.md) |
| `module-countpages` | CountPages Module | `documented` | [specification](specifications/module/module-countpages.md) |
| `module-createaccount` | Create Account Module | `invocation-only` | [specification](specifications/module/module-createaccount.md) |
| `module-css` | CSS Module | `documented` | [specification](specifications/module/module-css.md) |
| `module-currencyconvert` | Currency Convert Module | `invocation-only` | [specification](specifications/module/module-currencyconvert.md) |
| `module-dashboard` | Dashboard Module | `invocation-only` | [specification](specifications/module/module-dashboard.md) |
| `module-deleteaccount` | Delete Account Module | `invocation-only` | [specification](specifications/module/module-deleteaccount.md) |
| `module-featuredsite` | FeaturedSite Module | `documented` | [specification](specifications/module/module-featuredsite.md) |
| `module-feed` | Feed Module | `documented` | [specification](specifications/module/module-feed.md) |
| `module-files` | Files Module | `documented` | [specification](specifications/module/module-files.md) |
| `module-flickrgallery` | FlickrGallery Module | `documented` | [specification](specifications/module/module-flickrgallery.md) |
| `module-footerbar` | Footer Bar Module | `invocation-only` | [specification](specifications/module/module-footerbar.md) |
| `module-forumcategory` | Forum Category Module | `invocation-only` | [specification](specifications/module/module-forumcategory.md) |
| `module-forumnewthread` | Forum New Thread Module | `invocation-only` | [specification](specifications/module/module-forumnewthread.md) |
| `module-forumstart` | Forum Start Module | `invocation-only` | [specification](specifications/module/module-forumstart.md) |
| `module-forumthread` | Forum Thread Module | `invocation-only` | [specification](specifications/module/module-forumthread.md) |
| `module-frontforum` | FrontForum Module | `documented` | [specification](specifications/module/module-frontforum.md) |
| `module-frontspecialmini` | Front Special Mini Module | `invocation-only` | [specification](specifications/module/module-frontspecialmini.md) |
| `module-join` | Join Module | `documented` | [specification](specifications/module/module-join.md) |
| `module-listdrafts` | ListDrafts Module | `documented` | [specification](specifications/module/module-listdrafts.md) |
| `module-listpages` | ListPages Module | `documented` | [specification](specifications/module/module-listpages.md) |
| `module-listusers` | ListUsers Module | `documented` | [specification](specifications/module/module-listusers.md) |
| `module-loginstatus` | Login Status Module | `invocation-only` | [specification](specifications/module/module-loginstatus.md) |
| `module-mailform` | MailForm Module | `documented` | [specification](specifications/module/module-mailform.md) |
| `module-managesite` | ManageSite Module | `documented` | [specification](specifications/module/module-managesite.md) |
| `module-members` | Members Module | `documented` | [specification](specifications/module/module-members.md) |
| `module-membershipapply` | MembershipApply Module | `documented` | [specification](specifications/module/module-membershipapply.md) |
| `module-membershipbypassword` | MembershipByPassword Module | `documented` | [specification](specifications/module/module-membershipbypassword.md) |
| `module-membershipemailinvitation` | Membership Email Invitation Module | `invocation-only` | [specification](specifications/module/module-membershipemailinvitation.md) |
| `module-miniactivethreads` | MiniActiveThreads Module | `documented` | [specification](specifications/module/module-miniactivethreads.md) |
| `module-minirecentposts` | MiniRecentPosts Module | `documented` | [specification](specifications/module/module-minirecentposts.md) |
| `module-minirecentthreads` | MiniRecentThreads Module | `documented` | [specification](specifications/module/module-minirecentthreads.md) |
| `module-navibar` | Navi Bar Module | `invocation-only` | [specification](specifications/module/module-navibar.md) |
| `module-newpage` | NewPage Module | `documented` | [specification](specifications/module/module-newpage.md) |
| `module-newsite` | New Site Module | `invocation-only` | [specification](specifications/module/module-newsite.md) |
| `module-nextpreviouspage` | NextPreviousPage Module | `documented` | [specification](specifications/module/module-nextpreviouspage.md) |
| `module-orphanedpages` | OrphanedPages Module | `documented` | [specification](specifications/module/module-orphanedpages.md) |
| `module-pagecalendar` | PageCalendar Module | `documented` | [specification](specifications/module/module-pagecalendar.md) |
| `module-pageoptionsbottom` | Page Options Bottom Module | `invocation-only` | [specification](specifications/module/module-pageoptionsbottom.md) |
| `module-pages` | Pages Module | `documented` | [specification](specifications/module/module-pages.md) |
| `module-pagesbytag` | PagesByTag Module | `documented` | [specification](specifications/module/module-pagesbytag.md) |
| `module-pagetree` | PageTree Module | `documented` | [specification](specifications/module/module-pagetree.md) |
| `module-petitionadmin` | PetitionAdmin Module | `documented` | [specification](specifications/module/module-petitionadmin.md) |
| `module-rate` | Rate Module | `documented` | [specification](specifications/module/module-rate.md) |
| `module-ratedpages` | RatedPages Module | `documented` | [specification](specifications/module/module-ratedpages.md) |
| `module-recentposts` | RecentPosts Module | `documented` | [specification](specifications/module/module-recentposts.md) |
| `module-recentthreads` | Recent Threads Module | `invocation-only` | [specification](specifications/module/module-recentthreads.md) |
| `module-redirect` | Redirect Module | `documented` | [specification](specifications/module/module-redirect.md) |
| `module-search` | Search Module | `documented` | [specification](specifications/module/module-search.md) |
| `module-searchall` | SearchAll Module | `documented` | [specification](specifications/module/module-searchall.md) |
| `module-searchusers` | SearchUsers Module | `documented` | [specification](specifications/module/module-searchusers.md) |
| `module-sendinvitations` | SendInvitations Module | `documented` | [specification](specifications/module/module-sendinvitations.md) |
| `module-simpletodo` | SimpleToDo Module | `documented-deprecated` | [specification](specifications/module/module-simpletodo.md) |
| `module-sitechanges` | SiteChanges Module | `documented` | [specification](specifications/module/module-sitechanges.md) |
| `module-sitegrid` | SiteGrid Module | `documented` | [specification](specifications/module/module-sitegrid.md) |
| `module-sitestagcloud` | Sites Tag Cloud Module | `invocation-only` | [specification](specifications/module/module-sitestagcloud.md) |
| `module-tagcloud` | TagCloud Module | `documented` | [specification](specifications/module/module-tagcloud.md) |
| `module-themepreviewer` | ThemePreviewer Module | `documented` | [specification](specifications/module/module-themepreviewer.md) |
| `module-userinfo` | User Info Module | `invocation-only` | [specification](specifications/module/module-userinfo.md) |
| `module-wantedpages` | WantedPages Module | `documented` | [specification](specifications/module/module-wantedpages.md) |
| `module-watchers` | Watchers Module | `documented` | [specification](specifications/module/module-watchers.md) |
| `module-whoinvited` | WhoInvited Module | `documented` | [specification](specifications/module/module-whoinvited.md) |
| `account-lifecycle` | User account lifecycle and authentication recovery | `documented` | [specification](specifications/platform/account-lifecycle.md) |
| `advertising` | Site advertising | `documented` | [specification](specifications/platform/advertising.md) |
| `avatars` | User avatars | `high-level-documentation` | [specification](specifications/platform/avatars.md) |
| `browser-support` | Supported browsers | `documented` | [specification](specifications/platform/browser-support.md) |
| `collaborative-editing` | Collaborative page and file editing | `high-level-documentation` | [specification](specifications/platform/collaborative-editing.md) |
| `community-site-directory` | Community Site directory and application | `documented` | [specification](specifications/platform/community-site-directory.md) |
| `content-licensing` | Content licensing | `high-level-documentation` | [specification](specifications/platform/content-licensing.md) |
| `custom-domains` | Custom site domains | `high-level-documentation` | [specification](specifications/platform/custom-domains.md) |
| `educational-site-status` | Educational site status | `documented-plan-capability` | [specification](specifications/platform/educational-site-status.md) |
| `expressions` | Expressions | `documented` | [specification](specifications/platform/expressions.md) |
| `favicons` | Site favicons | `high-level-documentation` | [specification](specifications/platform/favicons.md) |
| `forum-signatures` | Forum signatures | `high-level-documentation` | [specification](specifications/platform/forum-signatures.md) |
| `gravatar` | Gravatar integration | `high-level-documentation` | [specification](specifications/platform/gravatar.md) |
| `hosted-wiki-platform` | Hosted wiki platform | `high-level-documentation` | [specification](specifications/platform/hosted-wiki-platform.md) |
| `karma` | User karma | `documented` | [specification](specifications/platform/karma.md) |
| `managed-hosting` | Managed site hosting | `high-level-documentation` | [specification](specifications/platform/managed-hosting.md) |
| `meta-tags` | Site and page metadata tags | `high-level-documentation` | [specification](specifications/platform/meta-tags.md) |
| `outgoing-pingbacks` | Outgoing pingbacks | `high-level-documentation` | [specification](specifications/platform/outgoing-pingbacks.md) |
| `page-editing-history` | Page editing modes and revision history | `documented` | [specification](specifications/platform/page-editing-history.md) |
| `page-templates` | Category page templates | `documented` | [specification](specifications/platform/page-templates.md) |
| `private-messages` | Private messages and contacts | `high-level-documentation` | [specification](specifications/platform/private-messages.md) |
| `private-sites` | Private sites | `documented` | [specification](specifications/platform/private-sites.md) |
| `roles-and-permissions` | Roles and permissions | `high-level-documentation` | [specification](specifications/platform/roles-and-permissions.md) |
| `search-language` | Search query language | `documented` | [specification](specifications/platform/search-language.md) |
| `secure-login` | Secure login | `high-level-documentation` | [specification](specifications/platform/secure-login.md) |
| `service-resilience` | Service resilience and data safety | `high-level-documentation` | [specification](specifications/platform/service-resilience.md) |
| `site-backups` | Site backups | `high-level-documentation` | [specification](specifications/platform/site-backups.md) |
| `site-cloning` | Site cloning | `high-level-documentation` | [specification](specifications/platform/site-cloning.md) |
| `site-https` | HTTPS site access | `high-level-documentation` | [specification](specifications/platform/site-https.md) |
| `site-lifecycle-limits` | Site limits, backup, anti-abuse, deletion, and restoration | `documented` | [specification](specifications/platform/site-lifecycle-limits.md) |
| `site-membership` | Site membership | `high-level-documentation` | [specification](specifications/platform/site-membership.md) |
| `site-navigation` | Site navigation | `high-level-documentation` | [specification](specifications/platform/site-navigation.md) |
| `site-storage` | Site file storage | `high-level-documentation` | [specification](specifications/platform/site-storage.md) |
| `site-themes` | Site themes | `high-level-documentation` | [specification](specifications/platform/site-themes.md) |
| `subscription-plan-matrix` | Subscription plan comparison | `documented` | [specification](specifications/platform/subscription-plan-matrix.md) |
| `subscriptions-plans` | Subscriptions and account/site plans | `documented` | [specification](specifications/platform/subscriptions-plans.md) |
| `syntax-engine` | Wiki syntax engine | `high-level-documentation` | [specification](specifications/platform/syntax-engine.md) |
| `thumbnails` | Page and site thumbnails | `documented` | [specification](specifications/platform/thumbnails.md) |
| `unlimited-pages` | Unlimited site pages | `high-level-documentation` | [specification](specifications/platform/unlimited-pages.md) |
| `user-roles` | Wikidot users and site roles | `documented` | [specification](specifications/platform/user-roles.md) |
| `watching-notifications` | Watching and email notifications | `documented` | [specification](specifications/platform/watching-notifications.md) |
| `web-statistics` | Web statistics | `high-level-documentation` | [specification](specifications/platform/web-statistics.md) |
| `content-pages` | Content pages | `documented` | [specification](specifications/site-structure/content-pages.md) |
| `forum-categories` | Forum categories | `documented` | [specification](specifications/site-structure/forum-categories.md) |
| `forum-category-groups` | Forum category groups | `documented` | [specification](specifications/site-structure/forum-category-groups.md) |
| `forum-posts` | Forum posts and post layout | `documented` | [specification](specifications/site-structure/forum-posts.md) |
| `forum-threads` | Forum threads | `documented` | [specification](specifications/site-structure/forum-threads.md) |
| `forums-overview` | Site forums | `documented` | [specification](specifications/site-structure/forums-overview.md) |
| `page-categories` | Page categories and namespaces | `documented` | [specification](specifications/site-structure/page-categories.md) |
| `page-forum-integration` | Page and forum integration | `documented` | [specification](specifications/site-structure/page-forum-integration.md) |
| `page-inclusions` | Page inclusion relationships | `documented` | [specification](specifications/site-structure/page-inclusions.md) |
| `page-links` | Direct page links | `documented` | [specification](specifications/site-structure/page-links.md) |
| `page-parent-relations` | Parent-page relations | `documented` | [specification](specifications/site-structure/page-parent-relations.md) |
| `page-tags` | Page tags | `documented` | [specification](specifications/site-structure/page-tags.md) |
| `site-identity` | Sites and site identity | `documented` | [specification](specifications/site-structure/site-identity.md) |
| `syntax-attachment` | Attached files syntax | `documented` | [specification](specifications/wiki-syntax/syntax-attachment.md) |
| `syntax-bibliography` | Bibliography syntax | `documented` | [specification](specifications/wiki-syntax/syntax-bibliography.md) |
| `syntax-block-formatting-elements` | Block Formatting Elements syntax | `documented` | [specification](specifications/wiki-syntax/syntax-block-formatting-elements.md) |
| `syntax-block-quotes` | Block Quotes syntax | `documented` | [specification](specifications/wiki-syntax/syntax-block-quotes.md) |
| `syntax-buttons` | Standalone buttons for page options syntax | `documented` | [specification](specifications/wiki-syntax/syntax-buttons.md) |
| `syntax-code-blocks` | Code Blocks syntax | `documented` | [specification](specifications/wiki-syntax/syntax-code-blocks.md) |
| `syntax-collapsible-blocks` | Collapsible Blocks syntax | `documented` | [specification](specifications/wiki-syntax/syntax-collapsible-blocks.md) |
| `syntax-comments` | Comments syntax | `documented` | [specification](specifications/wiki-syntax/syntax-comments.md) |
| `syntax-date` | Date syntax | `documented` | [specification](specifications/wiki-syntax/syntax-date.md) |
| `syntax-definition-lists` | Definition Lists syntax | `documented` | [specification](specifications/wiki-syntax/syntax-definition-lists.md) |
| `syntax-embedding` | Embedding media syntax | `documented` | [specification](specifications/wiki-syntax/syntax-embedding.md) |
| `syntax-embedding-code` | Embedding code from other sites syntax | `documented` | [specification](specifications/wiki-syntax/syntax-embedding-code.md) |
| `syntax-foldable-list` | Foldable List syntax | `documented` | [specification](specifications/wiki-syntax/syntax-foldable-list.md) |
| `syntax-footnotes` | Footnotes syntax | `documented` | [specification](specifications/wiki-syntax/syntax-footnotes.md) |
| `syntax-headings` | Headings syntax | `documented` | [specification](specifications/wiki-syntax/syntax-headings.md) |
| `syntax-horizontal-rules` | Horizontal Rules syntax | `documented` | [specification](specifications/wiki-syntax/syntax-horizontal-rules.md) |
| `syntax-html-blocks` | Html Blocks syntax | `documented` | [specification](specifications/wiki-syntax/syntax-html-blocks.md) |
| `syntax-iftags` | Iftags syntax | `documented` | [specification](specifications/wiki-syntax/syntax-iftags.md) |
| `syntax-images` | Images syntax | `documented` | [specification](specifications/wiki-syntax/syntax-images.md) |
| `syntax-include` | Include syntax | `documented` | [specification](specifications/wiki-syntax/syntax-include.md) |
| `syntax-inline-formatting` | Inline Formatting syntax | `documented` | [specification](specifications/wiki-syntax/syntax-inline-formatting.md) |
| `syntax-layout` | Layout elements syntax | `documented` | [specification](specifications/wiki-syntax/syntax-layout.md) |
| `syntax-links` | Links syntax | `documented` | [specification](specifications/wiki-syntax/syntax-links.md) |
| `syntax-lists` | Lists syntax | `documented` | [specification](specifications/wiki-syntax/syntax-lists.md) |
| `syntax-literal-text` | Literal Text syntax | `documented` | [specification](specifications/wiki-syntax/syntax-literal-text.md) |
| `syntax-math` | Math syntax | `documented` | [specification](specifications/wiki-syntax/syntax-math.md) |
| `syntax-notes` | Notes syntax | `documented` | [specification](specifications/wiki-syntax/syntax-notes.md) |
| `syntax-paragraphs-and-newline` | Paragraphs and newlines syntax | `documented` | [specification](specifications/wiki-syntax/syntax-paragraphs-and-newline.md) |
| `syntax-social-bookmarking` | Social Bookmarking syntax | `documented` | [specification](specifications/wiki-syntax/syntax-social-bookmarking.md) |
| `syntax-table-of-contents` | Table Of Contents syntax | `documented` | [specification](specifications/wiki-syntax/syntax-table-of-contents.md) |
| `syntax-tables` | Tables syntax | `documented` | [specification](specifications/wiki-syntax/syntax-tables.md) |
| `syntax-tag-buttons` | Button for tag update syntax | `documented` | [specification](specifications/wiki-syntax/syntax-tag-buttons.md) |
| `syntax-text-size` | Text Size syntax | `documented` | [specification](specifications/wiki-syntax/syntax-text-size.md) |
| `syntax-typography` | Typography syntax | `documented` | [specification](specifications/wiki-syntax/syntax-typography.md) |
| `syntax-universal-escaping` | Universal Escaping syntax | `documented` | [specification](specifications/wiki-syntax/syntax-universal-escaping.md) |
| `syntax-users` | Users syntax | `documented` | [specification](specifications/wiki-syntax/syntax-users.md) |
