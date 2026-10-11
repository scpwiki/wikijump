/**
 * Body of Wikidot's styleFrame.html
 * (https://interwiki.scpwiki.com/styleFrame.html, retrieved 2026-10-11). A
 * style frame only announces itself and pushes its own query string to
 * sibling interwiki frames that are already initialized; an interwiki
 * frame that initializes later pulls it instead. It never reads or writes
 * its parent document, and it renders nothing from its query string.
 */
export const WIKIDOT_STYLE_FRAME_SCRIPT = `
      "use strict";
      window.isStyleFrame = true;
      Array.prototype.slice.call(parent).forEach(function (frame) {
        try {
          if (frame.isInterwikiFrame) {
            frame.requestStyleChange(location.search);
          }
        } catch (error) {
          if (!(error instanceof DOMException)) {
            throw error;
          }
        }
      });
    `

/** @returns {string} */
export const buildWikidotStyleFrameHtml = () => `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Local Wikidot style frame</title>
    <script>${WIKIDOT_STYLE_FRAME_SCRIPT}</script>
  </head>
</html>
`
