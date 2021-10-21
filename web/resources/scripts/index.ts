// organize-imports-ignore

// TODO: remove this stupid hack, along with the other places where it's used
// Vite doesn't handle CSS/SCSS files being entrypoints AT ALL
// they simply won't get emitted to the manifest and are otherwise
// completely broken. the only way to get this to work is via this type of import.
// thankfully, when emitted for production, an actual stylesheet link is used
// so JS isn't needed.
import "./index.scss"

import "@wikijump/ftml-components"
import "../lib/account-panel"
import "../lib/elements/nav-dropdown"
import "../lib/elements/sidebar"
