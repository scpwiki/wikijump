# DEADCODE.md

Whenever code that was present in base Wikidot is removed from the repository,
make a note of it here. Record where it was, what it did, and why it was
removed.

If possible, add to this log in the same commit in which the code is removed.

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
