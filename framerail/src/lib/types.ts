export type Optional<T> = T | null
export type TranslateKeys = Record<string, Record<string, string | number>>
export type TranslatedKeys = Record<string, Optional<string>>
