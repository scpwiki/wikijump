export const dicts: Record<string, () => Promise<{ dict: string; bigram?: string }>> = {
  "en": async () => ({
    dict: await url(import("../../../../vendor/dicts/en-merged.txt?url"))
  }),
  "de": async () => ({
    dict: await url(import("../../../../vendor/dicts/de-100k.txt?url"))
  }),
  "es": async () => ({
    dict: await url(import("../../../../vendor/dicts/es-100l.txt?url"))
  }),
  "fr": async () => ({
    dict: await url(import("../../../../vendor/dicts/fr-100k.txt?url"))
  }),
  "he": async () => ({
    dict: await url(import("../../../../vendor/dicts/he-100k.txt?url"))
  }),
  "it": async () => ({
    dict: await url(import("../../../../vendor/dicts/it-100k.txt?url"))
  }),
  "ru": async () => ({
    dict: await url(import("../../../../vendor/dicts/ru-100k.txt?url"))
  }),
  "zh": async () => ({
    dict: await url(import("../../../../vendor/dicts/zh-50k.txt?url"))
  })
}

export default dicts

async function url(imp: Promise<any>) {
  return new URL((await imp).default, import.meta.url).toString()
}
