/* tslint:disable */
/* eslint-disable */
/**
*/
export class SymSpell {
  free(): void;
/**
* @param {any} parameters
*/
  constructor(parameters: any);
/**
* @param {Uint8Array} input
* @param {any} args
*/
  load_dictionary(input: Uint8Array, args: any): void;
/**
* @param {Uint8Array} input
* @param {any} args
*/
  load_bigram_dictionary(input: Uint8Array, args: any): void;
/**
* @param {string} input
* @param {number} edit_distance
* @returns {any[]}
*/
  lookup_compound(input: string, edit_distance: number): any[];
/**
* @param {string} input
* @param {number} verbosity
* @param {number} max_edit_distance
* @returns {any[]}
*/
  lookup(input: string, verbosity: number, max_edit_distance: number): any[];
/**
* @param {string} input
* @param {number} max_edit_distance
* @returns {any}
*/
  word_segmentation(input: string, max_edit_distance: number): any;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_symspell_free: (a: number) => void;
  readonly symspell_new: (a: number) => number;
  readonly symspell_load_dictionary: (a: number, b: number, c: number, d: number) => void;
  readonly symspell_load_bigram_dictionary: (a: number, b: number, c: number, d: number) => void;
  readonly symspell_lookup_compound: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly symspell_lookup: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly symspell_word_segmentation: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path: InitInput | Promise<InitInput>): Promise<InitOutput>;
