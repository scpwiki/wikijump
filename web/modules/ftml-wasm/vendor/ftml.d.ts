/* tslint:disable */
/* eslint-disable */
/**
* @param {Tokenization} tokens
* @param {PageInfo} page_info
* @param {WikitextSettings} settings
* @returns {ParseOutcome}
*/
export function parse(tokens: Tokenization, page_info: PageInfo, settings: WikitextSettings): ParseOutcome;
/**
* @param {SyntaxTree} syntax_tree
* @param {PageInfo} page_info
* @param {WikitextSettings} settings
* @returns {HtmlOutput}
*/
export function render_html(syntax_tree: SyntaxTree, page_info: PageInfo, settings: WikitextSettings): HtmlOutput;
/**
* @param {string} text
* @returns {string}
*/
export function preprocess(text: string): string;
/**
* @param {SyntaxTree} syntax_tree
* @param {PageInfo} page_info
* @param {WikitextSettings} settings
* @returns {string}
*/
export function render_text(syntax_tree: SyntaxTree, page_info: PageInfo, settings: WikitextSettings): string;
/**
* @returns {string}
*/
export function version(): string;
/**
* @param {string} text
* @returns {Tokenization}
*/
export function tokenize(text: string): Tokenization;


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





export interface IWikitextSettings {
    mode: WikitextMode;
    enable_page_syntax: boolean;
    use_true_ids: boolean;
    allow_local_paths: boolean;
}

export type WikitextMode =
    | 'page'
    | 'draft'
    | 'forum-post'
    | 'direct-message'
    | 'list'





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





export interface IToken {
    token: string;
    slice: string;
    span: {
        start: number;
        end: number;
    };
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
/**
*/
export class WikitextSettings {
  free(): void;
/**
* @returns {WikitextSettings}
*/
  copy(): WikitextSettings;
/**
* @param {IWikitextSettings} object
*/
  constructor(object: IWikitextSettings);
/**
* @param {string} mode
* @returns {WikitextSettings}
*/
  static from_mode(mode: string): WikitextSettings;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_parseoutcome_free: (a: number) => void;
  readonly parseoutcome_copy: (a: number) => number;
  readonly parseoutcome_syntax_tree: (a: number) => number;
  readonly parseoutcome_warnings: (a: number, b: number) => void;
  readonly __wbg_syntaxtree_free: (a: number) => void;
  readonly syntaxtree_data: (a: number, b: number) => void;
  readonly parse: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_wikitextsettings_free: (a: number) => void;
  readonly wikitextsettings_copy: (a: number) => number;
  readonly wikitextsettings_new: (a: number, b: number) => void;
  readonly wikitextsettings_from_mode: (a: number, b: number, c: number) => void;
  readonly syntaxtree_copy: (a: number) => number;
  readonly __wbg_pageinfo_free: (a: number) => void;
  readonly pageinfo_copy: (a: number) => number;
  readonly pageinfo_new: (a: number, b: number) => void;
  readonly pageinfo_page: (a: number, b: number) => void;
  readonly pageinfo_category: (a: number, b: number) => void;
  readonly pageinfo_site: (a: number, b: number) => void;
  readonly pageinfo_title: (a: number, b: number) => void;
  readonly pageinfo_alt_title: (a: number, b: number) => void;
  readonly pageinfo_rating: (a: number) => number;
  readonly pageinfo_tags: (a: number, b: number) => void;
  readonly pageinfo_language: (a: number, b: number) => void;
  readonly __wbg_htmloutput_free: (a: number) => void;
  readonly htmloutput_copy: (a: number) => number;
  readonly htmloutput_body: (a: number, b: number) => void;
  readonly htmloutput_styles: (a: number, b: number) => void;
  readonly htmloutput_html_meta: (a: number, b: number) => void;
  readonly htmloutput_backlinks: (a: number, b: number) => void;
  readonly render_html: (a: number, b: number, c: number) => number;
  readonly preprocess: (a: number, b: number, c: number) => void;
  readonly render_text: (a: number, b: number, c: number, d: number) => void;
  readonly version: (a: number) => void;
  readonly __wbg_tokenization_free: (a: number) => void;
  readonly tokenization_copy: (a: number) => number;
  readonly tokenization_text: (a: number, b: number) => void;
  readonly tokenization_tokens: (a: number, b: number) => void;
  readonly tokenize: (a: number, b: number) => number;
  readonly __wbg_utf16indexmap_free: (a: number) => void;
  readonly utf16indexmap_new: (a: number, b: number) => number;
  readonly utf16indexmap_get_index: (a: number, b: number, c: number) => void;
  readonly utf16indexmap_copy: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
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
