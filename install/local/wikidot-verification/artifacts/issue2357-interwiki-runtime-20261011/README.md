# Wikidot interwiki runtime sources (issue #2357)

Anonymous, read-only HTTPS GETs of the SCP interwiki runtime on 2026-10-11:

- `styleFrame.html`: https://interwiki.scpwiki.com/styleFrame.html
- `interwikiFrame.html`: https://interwiki.scpwiki.com/interwikiFrame.html?lang=en&community=scp&pagename=scp-9506
- `interwiki.js`: https://interwiki.scpwiki.com/interwiki.js
- `resizeIframe.js`: https://interwiki.scpwiki.com/resizeIframe.js

`framerail/src/lib/wikidot/wikidot-styleframe.js` (style frame) and
`framerail/src/lib/wikidot/wikidot-interwiki-style-runtime.js`
(`requestStyleChange`, priority merge, theme resolution, pull on init) are
ports of these sources. Theme and inline CSS apply only inside the interwiki
frame; neither source touches the embedding page.
