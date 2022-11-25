## framerail

Framerail is Wikijump's web server and client software, powered by SvelteKit.

### Development

Install node dependencies:

```bash
pnpm install
```

You can run a local instance using development mode:

```bash
pnpm dev

# or start the server and open the app in a new browser tab
pnpm dev -- --open
```

To create a production version of your app:

```bash
pnpm build
```

You can preview the production build with `pnpm preview`.

> To deploy your app, you may need to install an [adapter](https://kit.svelte.dev/docs/adapters) for your target environment.
