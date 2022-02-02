# DEADCODE.md

Whenever code that was present in base Wikidot is removed from the repository,
make a note of it here. Record where it was, what it did, and why it was
removed. See [WJ-218](https://scuttle.atlassian.net/browse/WJ-218) for more
details.

If possible, add to this log in the same commit in which the code is removed.

## PHP: SimpleToDo module
* Relevant Issues: [WJ-219](https://scuttle.atlassian.net/browse/WJ-219)
* Where it was: [SimpletodoListBase.php](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/php/db/base/DB/SimpletodoListBase.php), [SimpletodoListPeerBase.php](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/php/db/base/DB/SimpletodoListPeerBase.php), [SimpleToDoModule.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/modules/js/simpletodo/SimpleToDoModule.js)
* What it did: The `SimpleToDo` module usable in Wikitext, which provides a primitive editable list. See [Wikidot's documentation](https://www.wikidot.com/doc:simpletodo-module).
* Why it was removed: This feature is not used in real applications, is not a very well-designed UI for its task, and represents an unnecessary code maintenance burden.

## PHP: UnixifyString
* Relevant Issues: [WJ-236](https://scuttle.atlassian.net/browse/WJ-236)
* Where it was: [UnixifyString](https://github.com/scpwiki/wikijump/blob/571cd42cb810223a5dfe5f594b66adc39cb5295e/web/php/utils/Wikijump/Util/UnixifyString.php), [Unixify](https://github.com/scpwiki/wikijump/blob/571cd42cb810223a5dfe5f594b66adc39cb5295e/web/php/quickmodules/Unixify.php)
* What it did: It duplicated the code found in `WDStringUtils`, with some minor changes.
* Why it was removed: It was unused.

## JS: `OZONE.visuals.highlightText`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [OZONE.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/OZONE.js#L649)
* What it did: It was intended to highlight text in a page that matched a
  search result. It would have taken search terms from the URL path, probably
  from the sitewide search.
* Why it was removed: It was unused, except for a commented-out block of code
  in [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L897).
  It was also proving near-impossible to convert to Typescript without code
  gymnastics, and I don't think the feature is desirable in the first place.

## JS: `OZONE.dialogs.setButtons`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [OZONE.dialogs.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/dialog/OZONE.dialog.js#L564)
* What it did: Nothing. It would have been used to add multiple buttons to a
  pop-up dialogue. It will probably be re-added at some point.
* Why it was removed: It was an empty, unused function.

## JS: `OZONE.dialog.stock`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [OZONE.dialogs.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/dialog/OZONE.dialog.js#L564)
* What it did: A comment indicated that it was supposed to be an array of
  onscreen objects such as dialogue boxes and shaders. However, it is unused in
  favour of `OZONE.dialog.factory.stock`.
* Why it was removed: It was unused.

## JS: `OZONE.dialog.boxcontainer2.attachDD`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [OZONE.dialogs.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/dialog/OZONE.dialog.js#L167)
* What it did: Unsure. Something to do with enabling drag-and-drop on dialogue
  boxes. Unclear whether or not it worked as intended.
* Why it was removed: It was unused.

## JS: `Wikijump.utils.changeTextareaRowNo`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L14)
* What it did: Increased the height of a textarea by a number of lines.
* Why it was removed: It was unused.

## JS: `Wikijump.visuals`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L22)
* What it did: Contained a bunch of methods controlling the display of a
  "center-message" object. I believe this was a precursor to `OZONE.dialog`.
* Why it was removed: The object and all the methods in it were unused.

## JS: `Wikijump.utils.formatDates`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L83)
* What it did: Formatted dates within an element. Looks like it was a precursor
  to `OZONE.utils.formatDates`.
* Why it was removed: It was unused.

## JS: `Wikijump.render.fixAvatarHover`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L130)
* What it did: Controlled the avatar popup when a user's pic is hovered.
* Why it was removed: [WJ-224](https://scuttle.atlassian.net/browse/WJ-224)

## JS: `Wikijump.page.utils.scrollToReference`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L553)
* What it did: Alias for `OZONE.visuals.scrollTo`.
* Why it was removed: It was unused.

## JS: `Wikijump.page.utils.openHelpPop`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L571)
* What it did: Absolutely no idea. Opened a "HelpPop" page on an external wiki.
  Present in upstream, but doesn't do anything when called.
* Why it was removed: It was unused.

## JS: `Wikijump.page.listeners.loginClick0`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L132)
* What it did: Unsure. May have been an older version of `loginClick`.
* Why it was removed: Unused; referenced a nonexistent callback.

## JS: `OZONE.ajax.requestQuickModule`
* Relevant Issues: [WJ-250](https://scuttle.atlassian.net/browse/WJ-250)
* Where it was: [OZONE.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/OZONE.js#L124)
* What it did: Requested a "quickmodule" as opposed to a regular module.
  Made requests to `quickmodule.php`.  No indication what the difference
  between the two is.
* Why it was removed: It was unused, although the concept of a quick module
  appears on the PHP side of the codebase.

## PHP: `lib/ozoneframework/php/code/ListResolver.php`
* Relevant Issues: [WJ-189](https://scuttle.atlassian.net/browse/WJ-189)
* Where it was: `lib/ozoneframework/php/code/ListResolver.php`
* What it did: Strictly less than `lib/ozoneframework/php/template_services/ondemand/ListResolver.php` which was the intended file and Class to use.
* Why it was removed: The two files occupied the same namespace and caused Composer to complain when dumping the autoloader.

## PHP: `php/Screens/Misc/Captcha.php`
* Relevant Issues: [WJ-388](https://scuttle.atlassian.net/browse/WJ-388)
* Where it was: [Misc/Captcha.php](https://github.com/scpwiki/wikijump/blob/229d806a0fb13ee5af27317fd139257f05f6f4f6/web/php/Screens/Misc/Captcha.php)
* What it did: Wikidot's custom captcha implementation
* Why it was removed: Worked poorly, had security flaws, and was being replaced with a third-party captcha provider.

## PHP: `web/lib/zf`
* Relevant Issues: [WJ-428](https://scuttle.atlassian.net/browse/WJ-428)
* Where it was: [web/lib/zf/](https://github.com/scpwiki/wikijump/tree/3246cd5d72b7d4358b57eb9c78fd5515b7a39cb6/web/lib/zf)
* What it did: The entire Zend framework.
* Why it was removed: In addition to being vendor code in the repo, it was only used for search (broken) and pingbacks (obsolete).

## PHP: `web/php/Modules/CreateAccount`, `web/templates/modules/CreateAccount`, `web/web/files--common/modules/js/CreateAccount`
* Relevant Issues: (none)
* Where it was: [web/php/Modules/CreateAccount/](https://github.com/scpwiki/wikijump/tree/64b94cda1ff4e941da45621f0255bba19adae4ee/web/php/Modules/CreateAccount), [web/templates/modules/CreateAccount/](https://github.com/scpwiki/wikijump/tree/64b94cda1ff4e941da45621f0255bba19adae4ee/web/templates/modules/CreateAccount), [web/web/files--common/modules/js/CreateAccount/](https://github.com/scpwiki/wikijump/tree/64b94cda1ff4e941da45621f0255bba19adae4ee/web/web/files--common/modules/js/CreateAccount)
* What it did: The legacy `CreateAccount` module.
* Why it was removed: Unused.

## PHP: `web/php/DB/Invitation`, `web/php/DB/InvitationPeer`, `web/php/DB/AcceptedInvitation`, `web/php/DB/AcceptedInvitationPeer`, `web/php/DB/ApiKeyBase`, `web/php/DB/ApiKeyPeerBase`, `web/php/DB/CompletedMemberInvitation`, `web/php/DB/CompletedInMemberInvitationPeer`, `web/php/DB/ForumDivision`, `web/php/DB/ForumDivisionPeer`, `web/php/DB/ForumMessage`, `web/php/DB/ForumMessagePeer`, `web/php/DB/ForumTopic`, `web/php/DB/ForumTopicPeer`, `web/php/DB/ModuleCache`, `web/php/DB/ModuleCachePeer`, `web/php/Modules/Account/Elists/*`, `web/php/Actions/ManageSiteEmailListsAction`, `web/php/Modules/ManageSite/Elists/*`,  `web/lib/text_highlighter/*`, `web/lib/ozoneframework/php/core/IdBroker`, `web/lib/ozoneframework/php/core/DB/IdBrokerPeer`, `web/lib/ozoneframework/php/core/Database/DBGenerator*`, `web/lib/ozoneframework/php/core/Database/My*`, `web/php/DB/PageCompiledContent`, `web/php/DB/PageCompiledContentPeer`, `web/php/DB/PageFtsEntry`, `web/php/DB/PageFtsEntryPeer`, `web/php/DB/PageRate`, `web/php/DB/PageRatePeer`, `web/php/DB/PageSourceArchive`, `web/php/DB/PageSourceArchivePeer`, `web/web/private_file_filter.php`, `web/php/DB/ScreenCache`, `web/php/DB/ScreenCachePeer`, `web/php/Jobs/UpdateLuceneIndexJob`, `web/php/DB/UserSecurityQuestion`, `web/php/DB/UserSecurityQuestionPeer` 
* Where it was: As listed above, see [web/](https://github.com/scpwiki/wikijump/tree/64b94cda1ff4e941da45621f0255bba19adae4ee/web) directory.
* Relevant Issues: [WJ-436](https://scuttle.atlassian.net/browse/WJ-436)
* What it did: Nothing, all of these extended classes that did not appear in the codebase.
* Why it was removed: Unused code, throwing code inspection alerts.

## PHP and SQL: OpenID Support
* Where it was: All `seed.sql` files, `web/php/Actions/ManageSiteOpenIDAction`, `web/php/DB/Openid(Entry, Base, Peer, PeerBase)`, `web/php/Modules/ManageSite/ManageSiteOpenIDModule`, `web/templates/modules/ManageSite/ManageSiteOpenIDModule`, `web/web/files--common/modules/js/ManageSite/ManageSiteOpenIDModule`
* Relevant Issues: [WJ-314](https://scuttle.atlassian.net/browse/WJ-314)
* What it did: OpenID was apparently an IdP that was in greater use in 2008, a predecessor of sorts to OAuth. 
* Why it was removed: It has a number of unresolved security holes. We will be using alternate identity providers.

## PHP: `wikijump_token7`
* Where it was: [The AJAX module flow controller and cookie setters](https://github.com/scpwiki/wikijump/pull/193/files).
* Relevant Issues: [WJ-448](https://scuttle.atlassian.net/browse/WJ-448)
* What it did: Poorly-implemented [Cross Site Request Forgery (CSRF)](https://owasp.org/www-community/attacks/csrf) protection.
* Why it was removed: There are [recommended and more secure ways](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html) of guarding against CSRF attacks.

## PHP: `UniqueStrings::timeBased()`
* Where it was: [web/lib/ozoneframework/php/core/UniqueStrings.php](https://github.com/scpwiki/wikijump/blob/28b400207d15bbd89da2f273003098fac7ac541b/web/lib/ozoneframework/php/core/UniqueStrings.php)
* Relevant Issues: [WJ-442](https://scuttle.atlassian.net/browse/WJ-442)
* What it did: Produced a unique string based on the current time and an incrementing number.
* Why it was removed: Unused after an unnecessary usage was removed.

## PHP: Flickr integration
* Where it was: [Multiple files throughout the repository](https://github.com/scpwiki/wikijump/pull/256/files)
* Relevant Issues: [WJ-470](https://scuttle.atlassian.net/browse/WJ-470)
* What it did: Implemented an integration with an old version of Flickr's API, and added Flickr-related Wikidot features.
* Why it was removed: Mostly unused, outdated, and a large amount of code to maintain.

## PHP: Social Bookmarks integration
* Where it was: [web/php/Modules/Wiki/Social](https://github.com/scpwiki/wikijump/tree/6535307697de4e38a8c719d91ba71ab878c63820/web/php/Modules/Wiki/Social), [web/templates/modules/Wiki/Social](https://github.com/scpwiki/wikijump/tree/6535307697de4e38a8c719d91ba71ab878c63820/web/templates/modules/Wiki/Social), [web/web/files--common/images/social](https://github.com/scpwiki/wikijump/tree/6535307697de4e38a8c719d91ba71ab878c63820/web/web/files--common/images/social)
* Relevant Issues: [WJ-516](https://scuttle.atlassian.net/browse/WJ-516) (merged as part of [WJ-506](https://scuttle.atlassian.net/browse/WJ-506))
* What it did: Allowed users to add "social bookmarks" via services like del.icio.us.
* Why it was removed: Almost certainly broken, wasn't an active rule in Text\_Wiki, and half of the bookmark sites here are defunct.

## PHP: `WikidotAdmin/ManageUsersModule`
* Where it was: [web/php/Modules/WikidotAdmin/ManageUsers](https://github.com/scpwiki/wikijump/blob/19b276f3d56519eb717ebd52d5b7a940d415e7ac/web/php/Modules/WikidotAdmin/ManageUsersModule.php)
* Relevant Issues: [WJ-444](https://scuttle.atlassian.net/browse/WJ-444), [WJ-509](https://scuttle.atlassian.net/browse/WJ-509)
* What it did: It collects every single user in the database, presenting them as a single (unpaginated) table for a platform administrator to set permissions on.
* Why it was removed: `O(n)` performance on all users is very poor, and absolutely not scalable for Wikijump. Since the user refactor was occurring at the same time and it would've taken substantial work to fix this module, it was simply cut.

## PHP: `SiteSuperSettings`
* Where it was: [web/php/DB/SiteSuperSettings](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/SiteSuperSettings.php) and its associated database files.
* Relevant Issues: [WJ-759](https://scuttle.atlassian.net/browse/WJ-759), [WJ-221](https://scuttle.atlassian.net/browse/WJ-221)
* What it did: It was like the `site_settings` table, but a separate table for some reason.
* Why it was removed: Unnecessary table, and its only option (`can_custom_domain`) is unnecessary.

## PHP: `FtsEntry`
* Where it was [web/php/DB/FtsEntry](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/FtsEntry.php) and its associated database files.
* Relevant Issues: [WJ-725](https://scuttle.atlassian.net/browse/WJ-725), [WJ-889](https://scuttle.atlassian.net/browse/WJ-889)
* What it did: Managed full-text search via Postgres.
* Why it was removed: Search doesn't work, and we're fully replacing it anyways.

## PHP: `Ucookie`
* Where it was: [web/php/DB/Ucookie](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/Ucookie.php) and its associated database files.
* Relevant Issues: [WJ-765](https://scuttle.atlassian.net/browse/WJ-765)
* What it did: Unknown, apparently some sort of ancillary data attached to a particular session / site combo.
* Why it was removed: Unused.

## PHP, JS: `MailForm` module
* Where it was: [web/php/Actions/Wiki/MailFormAction](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/Actions/Wiki/MailFormAction.php), [web/php/Modules/Wiki/MailForm/MailFormModule](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/Modules/Wiki/MailForm/MailFormModule.php), [web/templates/modules/Wiki/MailForm/MailFormModule](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/templates/modules/Wiki/MailForm/MailFormModule.php)
* Relevant Issues: [WJ-762](https://scuttle.atlassian.net/browse/WJ-762)
* What it did: Allowed sending arbitrary emails to users, with content customized via its module syntax. See [its Wikidot documentation](https://www.wikidot.com/doc-modules:mailform-module).
* Why it was removed: Sole user of the "storage item" concept, and allowing users to send emails as Wikijump is a bad idea for several reasons. Will possibly reworked to have a smaller scope and be less freeform.

## PHP: `StorageItem` and `DatabaseStorage`
* Where it was: [web/php/DB/StorageItem](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/StorageItem.php), [web/php/Utils/DatabaseStorage](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/Utils/DatabaseStorage.php)
* Relevant Issues: [WJ-762](https://scuttle.atlassian.net/browse/WJ-762)
* What it did: Allowed storing arbitrary data for use in miscellaneous parts of the code.
* Why it was removed: Code smell, also only used by `MailForm` which is now gone (see above). Its corresponding database migrations were not in the code, so the table was non-functional.

## PHP: `PageTags`
* Where it was: [web/php/DB/PageTags](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/PageTag.php)
* Relevant Issues: [WJ-755](https://scuttle.atlassian.net/browse/WJ-755)
* What it did: Tag CRUD system, using the now-deprecated `page_tag` class.
* Why it was removed: Tags have been transferred to the `page` table, and all operations on the tags are now handled by Laravel.

## PHP: `LogEvent`
* Where it was: [web/php/DB/LogEvent](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/LogEvent.php)
* Relevant Issues: [WJ-730](https://scuttle.atlassian.net/browse/WJ-730)
* What it did: Wikidot's version of an audit log.
* Why it was removed: We're going to implement our own audit log, and there isn't anything particularly worth keeping in this implementation.

## PHP: `SiteBackup`, `Backuper`
* Where it was: [web/php/DB/SiteBackup](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/SiteBackup.php), [web/php/Utils/Backuper](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/Utils/Backuper.php)
* Relevant Issues: [WJ-757](https://scuttle.atlassian.net/browse/WJ-757), [WJ-896](https://scuttle.atlassian.net/browse/WJ-896)
* What it did: Produced backups of a site.
* Why it was removed: The backups have several flaws, the most notable of which is that they cannot be used to restore a site. Because is all essentially legacy code that will slow development, it has been removed.

## PHP: `OzoneLogger`
* Where it was: [web/lib/ozoneframework/php/core/OzoneLogger.php](https://github.com/scpwiki/wikijump/blob/f3be3f39545249c92e10e3a8e03b30b9cdecaa18/web/lib/ozoneframework/php/core/OzoneLogger.php)
* Relevant Issues: [WJ-895](https://scuttle.atlassian.net/browse/WJ-895)
* What it did: Basic logger
* Why it was removed: It didn't do anything that Laravel's logging didn't do better, and had some strange quirks that weren't worth keeping around. Additionally, Wikidot's code comments are not particularly high quality, and the same is also true of its logging. I prefixed all retained Ozone log calls with `[OZONE]` to make it easy to identify as legacy code.

## PHP: `DependencyFixer`
* Where it was: [web/php/Utils/DependencyFixer](https://github.com/scpwiki/wikijump/blob/f3be3f39545249c92e10e3a8e03b30b9cdecaa18/web/php/Utils/DependencyFixer.php)
* Relevant Issues: [WJ-920](https://scuttle.atlassian.net/browse/WJ-920)
* What it did: Edited pages linking to the page being renamed
* Why it was removed: It's a janky mess, we're redoing how pages are stored in the database, and there are various times when this feature isn't desirable, requiring more product consideration. (For instance, renaming an SCP article to `deleted:` and then having the series page auto-update to point there).

## PHP: `PageInclusion`, `PageLink`, `PageExternalLink`
* Where it was: [web/php/DB/PageInclusion.php](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/PageInclusion.php), [web/php/DB/PageLink.php](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/PageLink.php), [web/php/DB/PageExternalLink.php](https://github.com/scpwiki/wikijump/blob/d9a414d9319477673e23f1bbe16ad780394b0bb7/web/php/DB/PageExternalLink.php)
* Relevant Issues: [WJ-920](https://scuttle.atlassian.net/browse/WJ-920)
* What it did: Recorded included pages, backlinks, and external links.
* Why it was removed: As part of the page tables refactoring, it was replaced with tables `page_link` and `page_connection`, the latter being generic page-to-page connections with a type (e.g. include, link, etc).

## PHP: `PageSource`, `PageCompiled`
* Where it was: [web/php/DB/PageSource.php](https://github.com/scpwiki/wikijump/blob/4c8d379ec4ea78b99141c26e4d11ae466f87d04a/web/php/DB/PageSource.php), [web/php/DB/PageCompiled.php](https://github.com/scpwiki/wikijump/blob/4c8d379ec4ea78b99141c26e4d11ae466f87d04a/web/php/DB/PageCompiled.php)
* Relevant Issues: [WJ-920](https://scuttle.atlassian.net/browse/WJ-920)
* What it did: Stored the wikitext and compiled HTML for pages.
* Why it was removed: As part of the page tables refactoring, these were both replaced with a table `page_contents`, which stores both.

## PHP: `AnonymouseAbuseFlag`, `PageAbuseFlag`, `UserAbuseFlag`
* Where it was: [Multiple files, see pull request](https://github.com/scpwiki/wikijump/pull/612)
* Relevant Issues: [WJ-710](https://scuttle.atlassian.net/browse/WJ-710), [WJ-745](https://scuttle.atlassian.net/browse/WJ-745), [WJ-766](https://scuttle.atlassian.net/browse/WJ-766), [WJ-920](https://scuttle.atlassian.net/browse/WJ-920)
* What it did: Allowed users to flag content as objectionable.
* Why it was removed: Insufficient implementation. It was a binary toggle, without the ability to add reasons, and later additions such as easy-revoke are not present in this codebase. It will be easier to reimplement a sane abuse reporting system from scratch later.

## PHP: Section Editing
* Where it was: Multiple files, see pull request
* Relevant Issues: [WJ-936](https://scuttle.atlassian.net/browse/WJ-936)
* What it did: Allowed editing a portion of a page, separated by headings.
* Why it was removed: As described in the issue, it was buggy, not frequently used, and removing it simplifies page edit lock significantly as we engage in further refactoring.

## PHP: Append Editing
* Where it was: Multiple files, see pull request
* Relevant Issues: [WJ-938](https://scuttle.atlassian.net/browse/WJ-938)
* What it did: Allowed editing by adding contents to the end of a page.
* Why it was removed: It was frequently used, and with the removal of section editing, it made sense to simplify page edit code by removing it as well.

## PHP, JS: Page Edit Locks
* WHere it was: Multiple files, see pull request
* Relevant Issues: [WJ-747](https://scuttle.atlassian.net/browse/WJ-747)
* What it did: Implemented Wikidot's method of page edit locks.
* Why it was removed: As part of using our new editor, Sheaf, we will have a fundamentally different structure for managing locks. The schema for page tables is currently being refactored, so we are simplifying things by just removing it now.
