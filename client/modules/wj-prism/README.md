# wj-prism

This module wraps around the Prism syntax highlighting library.

> ### IMPORTANT
> The Prism library is vendored into this package. The file Prism resides in is mostly unmodified, except in two ways:
> 1. The `manual` property is hardcoded to `true`
> 2. The `disableWorkerMessageHandler` is hardcoded to `true`
>
> If the `prism.js` file is updated, e.g. with new languages, you must make the same edits to the file.
>
> These values will be found near the top of whatever Prism file you are editing.
