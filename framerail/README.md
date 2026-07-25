# Framerail

Framerail is Wikijump's SvelteKit web application and browser-facing compatibility layer. It turns HTTP requests and Deepwell results into rendered pages, form actions, legacy-compatible endpoints, static assets, and client-side interactions. Deepwell remains the owner of persistent data and trusted backend operations.

## Request and backend boundaries

The route/load/Deepwell boundary is a one-way dependency from route composition through application coordination to backend transport.

1. Files under `src/routes/` select a page or endpoint and compose its load function and actions.
2. Modules under `src/lib/server/load/` validate request data, assemble request context, coordinate a use case, and convert expected failures into SvelteKit responses.
3. Modules under `src/lib/server/deepwell/` expose typed operations and transport JSON-RPC requests. They do not depend on route or load modules.
4. Svelte components receive load data and action results. Browser-visible Wikidot behavior belongs in the relevant component or compatibility adapter, not in a generic post-render rewrite.

Page handling follows this boundary explicitly. `page.ts` loads page views and forms, while `page-edit-actions.ts`, `page-file-actions.ts`, `page-relation-actions.ts`, and `page-revision-actions.ts` own their corresponding validation and Deepwell coordination. The page route imports those capabilities directly and is their composition point.

## Browser compatibility

Compatibility adapters are the endpoint and rendering modules that preserve evidenced Wikidot behavior at Framerail's browser boundary. This includes legacy URL shapes such as `ajax-module-connector.php`, `xml-rpc-api.php`, and Wikidot interwiki frames; request metadata and locale handling; page action labels and interactions; DOM structure; security policy; and resource behavior.

Compatibility changes require evidence from live Wikidot or provenance-bearing corpus data. Matching normalized output is insufficient when author CSS, scripts, links, permissions, or network behavior can observe the difference. Sanitization, escaping, content security policy, and other security boundaries remain in force even when Wikidot behaves differently.

## Request hook responsibilities

`src/hooks.server.ts` applies cross-cutting work around route resolution. Before resolving a route it derives site, session, and page context and attempts eligible anonymous article-cache reads. During response generation it injects Wikidot request information. After resolution it applies static security headers and writes eligible anonymous article responses and cache tokens. Endpoint-specific business logic stays in routes and load modules.

`server.js` is the deployed HTTP entry point. It wraps the adapter-node handler with the anonymous article fast path, owns the in-memory fence-cache lifecycle, listens on `SOCKET_PATH` or `HOST` and `PORT`, and closes resources on termination signals.

## Where capabilities belong

| Change                                          | Owner                                                                                      |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Page or endpoint composition                    | `src/routes/`                                                                              |
| Request validation and application coordination | `src/lib/server/load/`                                                                     |
| Shared request-scoped context                   | `src/lib/server/request-context.ts`                                                        |
| Deepwell JSON-RPC operations                    | `src/lib/server/deepwell/`                                                                 |
| Authentication transport helpers                | `src/lib/server/auth/`                                                                     |
| Admin settings mapping                          | `src/lib/admin/`                                                                           |
| Browser layout state and shell selection        | `src/lib/layout/`                                                                          |
| Wikidot compatibility behavior                  | `src/lib/wikidot/`                                                                         |
| Global styles and theme primitives              | `src/lib/css/`                                                                             |
| Wikidot page shell and Sigma-style layout       | `src/lib/sigma-esque/`                                                                     |
| Route parameter matching                        | `src/params/`                                                                              |
| Node server and anonymous article fast path     | `server.js`, `article-response-fast-path.js`, and `src/lib/server/cache/article-response/` |
| Focused unit and integration tests              | `tests/`                                                                                   |

Add a capability to its existing owner before creating a new cross-cutting layer. A route should compose capabilities, a load module should coordinate them, and a Deepwell module should represent backend transport rather than browser policy.

## Local development

Install the locked dependencies:

```bash
pnpm install --frozen-lockfile
```

Start the development server:

```bash
pnpm dev
```

Run the standard checks:

```bash
pnpm lint
pnpm check
pnpm test
pnpm build
```

## Node deployment

Production uses `@sveltejs/adapter-node`, configured in `svelte.config.js`. Build the adapter output and start the repository server entry point:

```bash
pnpm build
node server.js
```

`pnpm preview` starts Vite's local preview server for inspection; it does not exercise the deployed `server.js` fast path or lifecycle.
