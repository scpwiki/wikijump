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
* @param {Tokenization} tokens
* @returns {ParseOutcome}
*/
export function parse(tokens: Tokenization): ParseOutcome;
/**
* @param {string} text
* @returns {string}
*/
export function preprocess(text: string): string;


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
