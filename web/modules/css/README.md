# @wikijump/css

This package follows the guidelines given by [sass-guidelin.es](https://sass-guidelin.es).

> ### NOTE:
> As it currently stands there are asset import issues - the gist is that you need to
> be careful if you're using `url()`. If you use it, make sure to use it in a `.css` file.
> Make sure to import that `.css` file from `@wikjump/css/src/...` so Vite can resolve
> where it is. Otherwise, it will try a relative import that will fail.
>
> See [`font-faces.css`](src/font-faces.css) and [`main.css`](src/main.scss) for an
> example of this workaround.
>
> This will be fixed at some point - but that'll probably require either compiling
> this package (rather than letting Vite do it) or upstream fixes from Vite.
> Currently, only JS imports (e.g. `import "@wikijump/css"`) correctly resolves `url()` paths.
