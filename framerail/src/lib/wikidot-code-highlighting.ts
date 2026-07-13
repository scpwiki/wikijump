import Prism from "prismjs"

const WIKIDOT_CODE_SELECTOR = ".code[data-wj-language]"
const MAX_HIGHLIGHT_CODE_UNITS = 100_000

type LanguageLoader = () => Promise<unknown>

const languageAliases: Record<string, string> = {
  "c#": "csharp",
  "c++": "cpp",
  cs: "csharp",
  dockerfile: "docker",
  dotnet: "csharp",
  html: "markup",
  js: "javascript",
  md: "markdown",
  py: "python",
  rb: "ruby",
  rss: "markup",
  sh: "bash",
  shell: "bash",
  svg: "markup",
  ts: "typescript",
  webmanifest: "json",
  xml: "markup",
  yml: "yaml"
}

const languageLoaders: Record<string, LanguageLoader> = {
  bash: () => import("prismjs/components/prism-bash.js"),
  c: () => import("prismjs/components/prism-c.js"),
  cpp: async () => {
    await languageLoaders.c()
    return import("prismjs/components/prism-cpp.js")
  },
  csharp: () => import("prismjs/components/prism-csharp.js"),
  diff: () => import("prismjs/components/prism-diff.js"),
  docker: () => import("prismjs/components/prism-docker.js"),
  go: () => import("prismjs/components/prism-go.js"),
  ini: () => import("prismjs/components/prism-ini.js"),
  java: () => import("prismjs/components/prism-java.js"),
  json: () => import("prismjs/components/prism-json.js"),
  jsx: () => import("prismjs/components/prism-jsx.js"),
  latex: () => import("prismjs/components/prism-latex.js"),
  less: () => import("prismjs/components/prism-less.js"),
  lua: () => import("prismjs/components/prism-lua.js"),
  markdown: () => import("prismjs/components/prism-markdown.js"),
  php: async () => {
    await import("prismjs/components/prism-markup-templating.js")
    return import("prismjs/components/prism-php.js")
  },
  powershell: () => import("prismjs/components/prism-powershell.js"),
  python: () => import("prismjs/components/prism-python.js"),
  ruby: () => import("prismjs/components/prism-ruby.js"),
  rust: () => import("prismjs/components/prism-rust.js"),
  scss: () => import("prismjs/components/prism-scss.js"),
  sql: () => import("prismjs/components/prism-sql.js"),
  toml: () => import("prismjs/components/prism-toml.js"),
  tsx: async () => {
    await Promise.all([languageLoaders.jsx(), languageLoaders.typescript()])
    return import("prismjs/components/prism-tsx.js")
  },
  typescript: () => import("prismjs/components/prism-typescript.js"),
  yaml: () => import("prismjs/components/prism-yaml.js")
}

interface HighlightState {
  html: string | null
  language: string
  source: string
}

const highlightStates = new WeakMap<HTMLElement, HighlightState>()
const highlightJobs = new WeakMap<HTMLElement, Promise<void>>()
const languageLoadJobs = new Map<string, Promise<boolean>>()

Prism.manual = true
Prism.hooks.add("wrap", (environment) => {
  environment.classes = environment.classes.map((className) => `wj-code-${className}`)
})

function canonicalLanguage(language: string): string {
  const normalized = language.trim().toLowerCase()
  return languageAliases[normalized] ?? normalized
}

async function loadLanguage(language: string): Promise<boolean> {
  if (Prism.languages[language]) return true
  const existingJob = languageLoadJobs.get(language)
  if (existingJob) return existingJob
  const loader = languageLoaders[language]
  if (!loader) return false

  const job = loader()
    .then(() => Boolean(Prism.languages[language]))
    .catch(() => false)
  languageLoadJobs.set(language, job)
  return job
}

export async function highlightWikidotCodeSource(
  source: string,
  requestedLanguage: string
): Promise<{ html: string; language: string } | null> {
  if (source.length > MAX_HIGHLIGHT_CODE_UNITS) return null
  const language = canonicalLanguage(requestedLanguage)
  if (["", "none", "plain", "plaintext", "raw", "text", "txt"].includes(language)) {
    return null
  }
  if (!(await loadLanguage(language))) return null
  const grammar = Prism.languages[language]
  if (!grammar || typeof grammar === "function") return null
  try {
    return { html: Prism.highlight(source, grammar, language), language }
  } catch {
    return null
  }
}

function isWikidotCodeBlock(element: Element): element is HTMLElement {
  return element.matches(WIKIDOT_CODE_SELECTOR)
}

export function highlightWikidotCodeBlock(block: HTMLElement): Promise<void> {
  const currentJob = highlightJobs.get(block)
  if (currentJob) return currentJob

  const job = (async () => {
    for (;;) {
      const code = block.querySelector("pre > code")
      const requestedLanguage = block.dataset.wjLanguage?.trim()
      if (!code || !requestedLanguage) return

      const source = code.textContent ?? ""
      const language = canonicalLanguage(requestedLanguage)
      const previous = highlightStates.get(block)
      if (previous?.language === language && previous.source === source) {
        if (previous.html !== null && code.innerHTML !== previous.html) {
          code.innerHTML = previous.html
        }
        return
      }

      const highlighted = await highlightWikidotCodeSource(source, requestedLanguage)
      if (
        block.dataset.wjLanguage?.trim() !== requestedLanguage ||
        block.querySelector("pre > code") !== code ||
        code.textContent !== source
      ) {
        continue
      }

      const html = highlighted?.html ?? null
      if (html !== null) code.innerHTML = html
      highlightStates.set(block, { html, language, source })
      return
    }
  })().finally(() => highlightJobs.delete(block))

  highlightJobs.set(block, job)
  return job
}

export function highlightWikidotCodeBlocks(root: ParentNode): Promise<void[]> {
  const blocks = new Set<HTMLElement>()
  if (root instanceof Element && isWikidotCodeBlock(root)) blocks.add(root)
  root.querySelectorAll(WIKIDOT_CODE_SELECTOR).forEach((element) => {
    if (element instanceof HTMLElement) blocks.add(element)
  })
  return Promise.all(Array.from(blocks, (block) => highlightWikidotCodeBlock(block)))
}

export function observeWikidotCodeBlocks(
  root: Document | Element = document
): () => void {
  void highlightWikidotCodeBlocks(root)

  const observationRoot = root instanceof Document ? root.documentElement : root
  if (!observationRoot) return () => {}

  const observer = new MutationObserver((records) => {
    const blocks = new Set<HTMLElement>()
    for (const record of records) {
      if (record.target instanceof Element) {
        const containingBlock = record.target.closest(WIKIDOT_CODE_SELECTOR)
        if (containingBlock instanceof HTMLElement) blocks.add(containingBlock)
      }
      record.addedNodes.forEach((node) => {
        if (!(node instanceof Element)) return
        if (isWikidotCodeBlock(node)) blocks.add(node)
        node.querySelectorAll(WIKIDOT_CODE_SELECTOR).forEach((block) => {
          if (block instanceof HTMLElement) blocks.add(block)
        })
      })
    }
    for (const block of blocks) void highlightWikidotCodeBlock(block)
  })
  observer.observe(observationRoot, { childList: true, subtree: true })
  return () => observer.disconnect()
}
