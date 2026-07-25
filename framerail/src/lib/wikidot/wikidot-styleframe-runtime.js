import {
  safeStyleFrameScriptJson,
  STYLEFRAME_MARKER
} from "./wikidot-styleframe-contract.js"
import { STYLEFRAME_INSERTION_RUNTIME_SOURCE } from "./wikidot-styleframe-runtime-insertion.js"
import { STYLEFRAME_ORDERING_RUNTIME_SOURCE } from "./wikidot-styleframe-runtime-ordering.js"
import { STYLEFRAME_OWNER_RUNTIME_SOURCE } from "./wikidot-styleframe-runtime-owner.js"

/**
 * @param {{ priority: string; themes: string[]; css: string }} input
 * @returns {string}
 */
export const buildWikidotStyleFrameRuntime = ({ priority, themes, css }) => {
  return `(() => {
  const marker = ${safeStyleFrameScriptJson(STYLEFRAME_MARKER)};
  const priority = ${safeStyleFrameScriptJson(priority)};
  const priorityNumber = Number.parseFloat(priority);
  const priorityValue = Number.isFinite(priorityNumber) ? priorityNumber : 0;
  const themes = ${safeStyleFrameScriptJson(themes)};
  const css = ${safeStyleFrameScriptJson(css)};
${STYLEFRAME_OWNER_RUNTIME_SOURCE}
${STYLEFRAME_ORDERING_RUNTIME_SOURCE}
${STYLEFRAME_INSERTION_RUNTIME_SOURCE}
})();`
}
