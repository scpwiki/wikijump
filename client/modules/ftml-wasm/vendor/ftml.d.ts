/* tslint:disable */
/* eslint-disable */
/**
* @param {PageInfo} page_info
* @param {SyntaxTree} syntax_tree
* @returns {HtmlOutput}
*/
export function render_html(page_info: PageInfo, syntax_tree: SyntaxTree): HtmlOutput;
/**
* @param {PageInfo} page_info
* @param {SyntaxTree} syntax_tree
* @returns {string}
*/
export function render_text(page_info: PageInfo, syntax_tree: SyntaxTree): string;
/**
* @param {string} text
* @returns {Tokenization}
*/
export function tokenize(text: string): Tokenization;
/**
* @returns {string}
*/
export function version(): string;
/**
* @param {string} text
* @returns {string}
*/
export function preprocess(text: string): string;
/**
* @param {Tokenization} tokens
* @returns {ParseOutcome}
*/
export function parse(tokens: Tokenization): ParseOutcome;


export interface IHtmlOutput {
    body: string;
    style: string;
    meta: IHtmlMeta[];
}

export interface IHtmlMeta {
    tag_type: string;
    name: string;
    value: string;
}

export interface IBacklinks {
    included_pages: string[];
    internal_links: string[];
    external_links: string[];
}

export interface IPageInfo {
    page: string;
    category: string | null;
    site: string;
    title: string;
    alt_title: string | null;
    rating: number;
    tags: string[];
    language: string;
}





export interface IToken {
    token: string;
    slice: string;
    span: {
        start: number;
        end: number;
    };
}





export interface IElement {
    element: string;
    data?: any;
}

export interface ISyntaxTree {
    elements: IElement[];
    styles: string[];
}

export interface IParseWarning {
    token: string;
    rule: string;
    span: {
        start: number;
        end: number;
    };
    kind: string;
}



/**
*/
export class HtmlOutput {
  free(): void;
/**
* @returns {HtmlOutput}
*/
  copy(): HtmlOutput;
/**
* @returns {string}
*/
  body(): string;
/**
* @returns {string[]}
*/
  styles(): string[];
/**
* @returns {IHtmlMeta[]}
*/
  html_meta(): IHtmlMeta[];
/**
* @returns {IBacklinks}
*/
  backlinks(): IBacklinks;
}
/**
*/
export class PageInfo {
  free(): void;
/**
* @returns {PageInfo}
*/
  copy(): PageInfo;
/**
* @param {IPageInfo} object
*/
  constructor(object: IPageInfo);
/**
* @returns {string | undefined}
*/
  readonly alt_title: string | undefined;
/**
* @returns {string | undefined}
*/
  readonly category: string | undefined;
/**
* @returns {string}
*/
  readonly language: string;
/**
* @returns {string}
*/
  readonly page: string;
/**
* @returns {number}
*/
  readonly rating: number;
/**
* @returns {string}
*/
  readonly site: string;
/**
* @returns {string[]}
*/
  readonly tags: string[];
/**
* @returns {string}
*/
  readonly title: string;
}
/**
*/
export class ParseOutcome {
  free(): void;
/**
* @returns {ParseOutcome}
*/
  copy(): ParseOutcome;
/**
* @returns {SyntaxTree}
*/
  syntax_tree(): SyntaxTree;
/**
* @returns {IParseWarning[]}
*/
  warnings(): IParseWarning[];
}
/**
*/
export class SyntaxTree {
  free(): void;
/**
* @returns {SyntaxTree}
*/
  copy(): SyntaxTree;
/**
* @returns {ISyntaxTree}
*/
  data(): ISyntaxTree;
}
/**
*/
export class Tokenization {
  free(): void;
/**
* @returns {Tokenization}
*/
  copy(): Tokenization;
/**
* @returns {string}
*/
  text(): string;
/**
* @returns {IToken[]}
*/
  tokens(): IToken[];
}
/**
*/
export class Utf16IndexMap {
  free(): void;
/**
* @param {string} text
*/
  constructor(text: string);
/**
* @returns {Utf16IndexMap}
*/
  copy(): Utf16IndexMap;
/**
* @param {number} index
* @returns {number}
*/
  get_index(index: number): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_pageinfo_free: (a: number) => void;
  readonly pageinfo_new: (a: number) => number;
  readonly pageinfo_page: (a: number, b: number) => void;
  readonly pageinfo_category: (a: number, b: number) => void;
  readonly pageinfo_site: (a: number, b: number) => void;
  readonly pageinfo_title: (a: number, b: number) => void;
  readonly pageinfo_alt_title: (a: number, b: number) => void;
  readonly pageinfo_rating: (a: number) => number;
  readonly pageinfo_tags: (a: number) => number;
  readonly pageinfo_language: (a: number, b: number) => void;
  readonly __wbg_htmloutput_free: (a: number) => void;
  readonly htmloutput_copy: (a: number) => number;
  readonly htmloutput_body: (a: number, b: number) => void;
  readonly htmloutput_styles: (a: number) => number;
  readonly htmloutput_html_meta: (a: number) => number;
  readonly htmloutput_backlinks: (a: number) => number;
  readonly render_html: (a: number, b: number) => number;
  readonly render_text: (a: number, b: number, c: number) => void;
  readonly pageinfo_copy: (a: number) => number;
  readonly __wbg_utf16indexmap_free: (a: number) => void;
  readonly utf16indexmap_new: (a: number, b: number) => number;
  readonly utf16indexmap_copy: (a: number) => number;
  readonly utf16indexmap_get_index: (a: number, b: number) => number;
  readonly __wbg_tokenization_free: (a: number) => void;
  readonly tokenization_copy: (a: number) => number;
  readonly tokenization_text: (a: number, b: number) => void;
  readonly tokenization_tokens: (a: number) => number;
  readonly tokenize: (a: number, b: number) => number;
  readonly version: (a: number) => void;
  readonly preprocess: (a: number, b: number, c: number) => void;
  readonly __wbg_parseoutcome_free: (a: number) => void;
  readonly parseoutcome_copy: (a: number) => number;
  readonly parseoutcome_syntax_tree: (a: number) => number;
  readonly parseoutcome_warnings: (a: number) => number;
  readonly __wbg_syntaxtree_free: (a: number) => void;
  readonly syntaxtree_data: (a: number) => number;
  readonly parse: (a: number) => number;
  readonly syntaxtree_copy: (a: number) => number;
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
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
