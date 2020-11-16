# DEADCODE.md

Whenever code that was present in base Wikidot is removed from the repository,
make a note of it here. Record where it was, what it did, and why it was
removed. See [WJ-218](https://scuttle.atlassian.net/browse/WJ-218) for more
details.

If possible, add to this log in the same commit in which the code is removed.

## PHP: SimpleToDo module
* Relevant Issues: [WDBUGS-219](https://scuttle.atlassian.net/browse/WDBUGS-219)
* Where it was: [SimpletodoListBase.php](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/php/db/base/DB/SimpletodoListBase.php), [SimpletodoListPeerBase.php](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/php/db/base/DB/SimpletodoListPeerBase.php), [SimpleToDoModule.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/modules/js/simpletodo/SimpleToDoModule.js)
* What it did: The `SimpleToDo` module usable in Wikitext, which provides a primitive editable list. See [Wikidot's documentation](https://www.wikidot.com/doc:simpletodo-module).
* Why it was removed: This feature is not used in real applications, is not a very well-designed UI for its task, and represents an unnecessary code maintenance burden.

## PHP: UnixifyString
* Relevant Issues: [WDBUGS-236](https://scuttle.atlassian.net/browse/WDBUGS-236)
* Where it was: [UnixifyString](https://github.com/scpwiki/wikijump/blob/571cd42cb810223a5dfe5f594b66adc39cb5295e/web/php/utils/Wikijump/Util/UnixifyString.php), [Unixify](https://github.com/scpwiki/wikijump/blob/571cd42cb810223a5dfe5f594b66adc39cb5295e/web/php/quickmodules/Unixify.php)
* What it did: It duplicated the code found in `WDStringUtils`, with some minor changes.
* Why it was removed: It was unused.

## JS: `OZONE.visuals.highlightText`
* Where it was: [OZONE.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/OZONE.js#L649)
* What it did: It was intended to highlight text in a page that matched a
  search result. It would have taken search terms from the URL path, probably
  from the sitewide search.
* Why it was removed: It was unused, except for a commented-out block of code
  in [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L897).
  It was also proving near-impossible to convert to Typescript without code
  gymnastics, and I don't think the feature is desirable in the first place.

## JS: `OZONE.dialogs.setButtons`
* Where it was: [OZONE.dialogs.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/dialog/OZONE.dialog.js#L564)
* What it did: Nothing. It would have been used to add multiple buttons to a
  pop-up dialogue. It will probably be re-added at some point.
* Why it was removed: It was an empty, unused function.

## JS: `OZONE.dialog.stock`
* Where it was: [OZONE.dialogs.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/dialog/OZONE.dialog.js#L564)
* What it did: A comment indicated that it was supposed to be an array of
  onscreen objects such as dialogue boxes and shaders. However, it is unused in
  favour of `OZONE.dialog.factory.stock`.
* Why it was removed: It was unused.

## JS: `OZONE.dialog.boxcontainer2.attachDD`
* Where it was: [OZONE.dialogs.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/dialog/OZONE.dialog.js#L167)
* What it did: Unsure. Something to do with enabling drag-and-drop on dialogue
  boxes. Unclear whether or not it worked as intended.
* Why it was removed: It was unused.

## JS: `Wikijump.utils.changeTextareaRowNo`
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L14)
* What it did: Increased the height of a textarea by a number of lines.
* Why it was removed: It was unused.

## JS: `Wikijump.visuals`
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L22)
* What it did: Contained a bunch of methods controlling the display of a
  "center-message" object. I believe this was a precursor to `OZONE.dialog`.
* Why it was removed: The object and all the methods in it were unused.

## JS: `Wikijump.utils.formatDates`
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L83)
* What it did: Formatted dates within an element. Looks like it was a precursor
  to `OZONE.utils.formatDates`.
* Why it was removed: It was unused.

## JS: `Wikijump.render.fixAvatarHover`
* Where it was: [Wikijump.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.js#L130)
* What it did: Controlled the avatar popup when a user's pic is hovered.
* Why it was removed:
  [WDBUGS-224](https://scuttle.atlassian.net/browse/WDBUGS-224)

## JS: `Wikijump.page.utils.scrollToReference`
* Where it was: [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L553)
* What it did: Alias for `OZONE.visuals.scrollTo`.
* Why it was removed: It was unused.

## JS: `Wikijump.page.utils.openHelpPop`
* Where it was: [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L571)
* What it did: Absolutely no idea. Opened a "HelpPop" page on an external wiki.
  Present in upstream, but doesn't do anything when called.
* Why it was removed: It was unused.

## JS: `Wikijump.page.listeners.loginClick0`
* Where it was: [Wikijump.page.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/Wikijump.page.js#L132)
* What it did: Unsure. May have been an older version of `loginClick`.
* Why it was removed: Unused; referenced a nonexistent callback.

## JS: `OZONE.ajax.requestQuickModule`
* Where it was: [OZONE.js](https://github.com/scpwiki/wikijump/blob/439c92376f04adaf73af87e2f53edabced8ca90f/web/files--common/javascript/OZONE.js#L124)
* What it did: Requested a "quickmodule" as opposed to a regular module.
  Made requests to `quickmodule.php`.  No indication what the difference
  between the two is.
* Why it was removed: It was unused, although the concept of a quick module
  appears on the PHP side of the codebase.
