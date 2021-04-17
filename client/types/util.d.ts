/**
 * @file Misc. type declarations utilized across the repo.
 */

// TODO: determine if this file & folder is a good idea

export declare global {
  /** Represents any function, without using the {@link Function} object. */
  type AnyFunction<T = unknown> = (...args: any) => T

  /** Represents the eventual value of a `Promise`. */
  export type PromiseValue<
    PromiseType,
    Otherwise = PromiseType
  > = PromiseType extends Promise<infer Value>
    ? { 0: PromiseValue<Value>; 1: Value }[PromiseType extends Promise<unknown> ? 0 : 1]
    : Otherwise

  /** All JS primitive values. */
  type Primitive = string | number | bigint | boolean | symbol | null | undefined

  /** _Strictly_ represents a `{ 'key': value }` object with only primitives. */
  type PlainObject = Record<string, Primitive>

  /** Represents either the value `T` or the value wrapped in `PromiseLike<T>`. */
  type Promisable<T> = T | PromiseLike<T>

  /** Matches a JSON object. */
  type JSONObject = { [Key in string]?: JSONValue }

  /** Matches a JSON array. */
  type JSONArray = JSONValue[]

  /** Matches any valid JSON value. */
  type JSONValue = string | number | boolean | null | JSONObject | JSONArray

  // Fixes import.meta.env for Snowpack
  interface ImportMeta {
    env: Record<string, string>
  }
}
