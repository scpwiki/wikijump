/*
 * parse.rs
 *
 * wikidot-html - Library to convert Wikidot syntax into HTML
 * Copyright (c) 2019 Ammon Smith for Project Foundation
 *
 * wikidot-html is available free of charge under the terms of the MIT
 * License. You are free to redistribute and/or modify it under those
 * terms. It is distributed in the hopes that it will be useful, but
 * WITHOUT ANY WARRANTY. See the LICENSE file for more details.
 *
 */

use regex::{Regex, RegexBuilder};

pub fn build_regex(pattern: &str, flags: &str) -> Regex {
    let mut regex = RegexBuilder::new(pattern);

    for ch in flags.chars() {
        match ch {
            'i' => { regex.case_insensitive(true); },
            'm' => { regex.multi_line(true); },
            's' => { regex.dot_matches_new_line(true); },
            'U' => { regex.swap_greed(true); },
            'x' => { regex.ignore_whitespace(true); },
            _ => panic!("Unknown regex flag: '{}'", ch),
        }
    }

    regex.build().expect("Parsing regular expression failed")
}

lazy_static! {
    pub static ref BIBLIOGRAPHY_CITE: Regex = build_regex(
        r"\(\(bibcite\s([a-z0-9]+)\)\)",
        "i",
    );

    pub static ref BIBLIOGRAPHY: Regex = build_regex(
        r"^\[\[bibliography(\s+[^\]]+)?\]\](.*?)\[\[\/bibliography\]\][ ]*$",
        "sm",
    );
}

lazy_static! {
    pub static ref BREAK: Regex = build_regex(
        r" _\n",
        "",
    );

    pub static ref BUTTON: Regex = build_regex(
        r"\[\[button\s+([a-z0-9\-_]+)(?:\s+(.+?))?\]\]",
        "is",
    );

    pub static ref CENTER: Regex = build_regex(
        r"\n\= (.*?)\n",
        "",
    );
}

lazy_static! {
    pub static ref CLEAR_FLOAT: Regex = build_regex(
        r"^([~]{4,})(>|<)?$",
        "m",
    );

    pub static ref CODE: Regex = build_regex(
        r"^\[\[code(\s[^\]]*)?\]\]((?:(?R)|.)*?)\[\[/code\]\](\s|$)",
        "msi",
    );

    pub static ref COLLAPSIBLE: Regex = build_regex(
        r"(\n)?\[\[collapsible(\s.*?)?\]\](.*?)\[\[\/collapsible\]\] *",
        "msi",
    );
}

lazy_static! {
    pub static ref CSS: Regex = build_regex(
        r"^\[\[css\]\]((?:(?R)|.)*?)\[\[/css\]\](\s|$)",
        "msi",
    );

    pub static ref DATE: Regex = build_regex(
        r"\[\[date\s+([0-9]+)(\s+.*?)?\]\]",
        "",
    );

    pub static ref DEFINITION_LIST: Regex = build_regex(
        r"\n((: ).*\n)(?!(: |\n))",
        "Us",
    );
}

lazy_static! {
    pub static ref DIV_ALIGN: Regex = build_regex(
        r"^\[\[(=|<|>|==)\]\]\n((?:(?R)|.)*?)\[\[\/\\1\]\]$",
        "msi",
    );

    pub static ref DIV_PRE_FILTER: Regex = build_regex(
        r"\[\[\/div\]\](\s*?)\[\[div",
        "msi",
    );

    pub static ref EMBED: Regex = build_regex(
        r"\[\[embed(?:audio|video)?\]\](.*?)\[\[\/embed(?:audio|video)?\]\]",
        "msi",
    );
}

lazy_static! {
    pub static ref PARSE_EMPHASIS: Regex = build_regex(
        r"\/\/([^\s](?:.*[^\s])?)\/\/",
        "U",
    );

    pub static ref PARSE_EQUATION_REFERENCE: Regex = build_regex(
        r"\[\[eref (.*?)\]\]",
        "",
    );

    pub static ref PARSE_FILE: Regex = build_regex(
        r"\[\[file\s+(.+?)(?:\|(.+?))?\]\]",
        "i",
    );

    pub static ref PARSE_FOOTNOTE: Regex = build_regex(
        r"\s*\[\[footnote\]\](.*?)\[\[/footnote\]\]",
        "s",
    );
}

lazy_static! {
    pub static ref PARSE_FOOTNOTE_BLOCK: Regex = build_regex(
        r"(\[\[footnoteblock(\s+[^\]]+)?\]\])|($)",
        "s",
    );

    pub static ref PARSE_FORM: Regex = build_regex(
        r"\[\[form\]\]\s*\n(.*)\n---\s*\n(.*)\n\[\[\/form\]\]",
        "is",
    );

    pub static ref PARSE_FREE_LINK: Regex = build_regex(
        r"\[\[\[([^\]\|\[#]+)\s*(#[A-Za-z][-A-Za-z0-9_:.]*)?\s*(\|[^\]\|\[\#]*)?()\]\]\]",
        "",
    );

    pub static ref PARSE_FUNCTION: Regex = build_regex(
        r"^(\<function\>)\n(.+)\n(\<\/function\>)(\s|$)",
        "Umsi",
    );
}

lazy_static! {
    pub static ref PARSE_GALLERY: Regex = build_regex(
        r"^\[\[gallery(\s[^\]]*?)?\]\](?:((?:\n: [^\n]+)+)\n\[\[\/gallery\]\])?",
        "msi",
    );

    pub static ref PARSE_HEADING: Regex = build_regex(
        r"^(\+{1,6}) (.*)",
        "m",
    );

    pub static ref PARSE_HORIZONTAL: Regex = build_regex(
        r"^([-]{4,})$",
        "m",
    );

    pub static ref PARSE_HTML: Regex = build_regex(
        r"^\<html\>\n(.+)\n\<\/html\>(\s|$)",
        "Umsi",
    );
}

lazy_static! {
    pub static ref PARSE_IFTAGS: Regex = build_regex(
        r"[\[iftags(\s[^\]]*)?\]\]((?:(?R)|.)*?)\[\[/iftags\]\]",
        "msi",
    );

    pub static ref PARSE_IMAGE: Regex = build_regex(
        r"(\[\[((?:f)?[<>=])?image\s+)(.+?)(?:\]\])(?:(.*?)\[\[\/image\]\])?",
        "is",
    );

    pub static ref PARSE_INCLUDE: Regex = build_regex(
        r"^\[\[include ([a-zA-Z0-9\s\-:]+?)(\s+.*?)?(?:\]\])$",
        "ims",
    );

    pub static ref PARSE_INTERWIKI: Regex = build_regex(
        r"([A-Za-z0-9_\.]+):((?!:)[A-Za-z0-9_\/=&~#.:;\-\+]+)",
        "",
    );
}

lazy_static! {
    pub static ref PARSE_ITALIC: Regex = build_regex(
        r"\/\/([^\/].*\/\/",
        "U",
    );

    pub static ref PARSE_LIST: Regex = build_regex(
        r"^((\*|#) .*\n)(?!\2 |(?: {1,}((?:\*|#) |\n)))",
        "Usm",
    );

    pub static ref PARSE_MATH: Regex = build_regex(
        r#"#^\[\[math(\s+[a-z0-9_]*?)?((?:\s+[a-z0-9]+="[^"]*"))*\s*\]\]((?:(?R)|.)*?)\n\[\[/math\]\](\s|$)#"#,
        "msi",
    );

    pub static ref PARSE_MATH_INLINE: Regex = build_regex(
        r"\[\[\$(.*?)\$\]\]",
        "",
    );
}

lazy_static! {
    pub static ref PARSE_MODULE: Regex = build_regex(
        r"^\[\[module\s([a-z0-9_\-\/]+)(\s+.*?)?\]\] *\n(?:(.*?)\[\[\/module\]\])?",
        "msi",
    );

    pub static ref PARSE_MODULE_654: Regex = build_regex(
        r"^\[\[module654\s([a-z0-9_\-\/]+)(\s+.*?)?\]\]\n(?:(.*?)\[\[\/module\]\])?",
        "msi",
    );

    pub static ref PARSE_MODULE_PRE: Regex = build_regex(
        r"^\[\[module654\s([a-z0-9_\-\/]+)(\s+.*?)?\]\](?:(.*?)\[\[\/module\]\])?",
        "msi",
    );

    pub static ref PARSE_NEWLINE: Regex = build_regex(
        r"([^\n])\n(?!\n)",
        "m",
    );
}

lazy_static! {
    pub static ref PARSE_NOTE: Regex = build_regex(
        r"(\n)?\[\[note\]\]\n(.*?)\[\[\/note\]\]",
        "msi",
    );

    pub static ref PARSE_PARAGRAPH: Regex = build_regex(
        r"^.*?\n\s*?\n",
        "m",
    );

    pub static ref PARSE_PHP: Regex = build_regex(
        r"\[\[php (.+?)\]\]",
        "",
    );

    pub static ref PARSE_RAW: Regex = build_regex(
        r"@@(.*[^@]?)@@",
        "U",
    );
}

lazy_static! {
    pub static ref PARSE_SEPARATOR: Regex = build_regex(
        r"^(={4,})$",
        "m",
    );

    pub static ref PARSE_SIZE: Regex = build_regex(
        r"\[\[size\s([^\]]+)\]\]((?:(?R)|.)*?)\[\[\/size\]\]",
        "msi",
    );

    pub static ref PARSE_SOCIAL: Regex = build_regex(
        r"\[\[social(\s+[^\]]+?)?(?:\]\])",
        "is",
    );

    pub static ref PARSE_SPAN: Regex = build_regex(
        r"\[\[span(\s.[^\]]*)?\]\]((?:(?R)|.)*?)\[\[\/span\]\]",
        "msi",
    );
}

lazy_static! {
    pub static ref PARSE_STRIKETHROUGH: Regex = build_regex(
        r"--([^\s](?:.*[^\s])?)--",
        "U",
    );

    pub static ref PARSE_STRONG: Regex = build_regex(
        r"\*\*([^\s\n](?:.*[^\s\n])?)\*\*",
        "U",
    );

    pub static ref PARSE_SUBSCRIPT: Regex = build_regex(
        r",,([^\s](?:.*[^\s])?),,",
        "U",
    );

    pub static ref PARSE_SUPERSCRIPT: Regex = build_regex(
        r"\^\^([^\s](?:.*[^\s])?)\^\^",
        "U",
    );
}

lazy_static! {
    pub static ref PARSE_TABLE: Regex = build_regex(
        r"\n((\|\|).*)(\n)(?!(\|\|))",
        "Us",
    );

    pub static ref PARSE_TABLE_ADV: Regex = build_regex(
        r"\n\[\[table(\s.*?)?\]\](\s*(?:\[\[row(?:\s[^\]]*)?\]\]\s*(?:\[\[(column|col|cell)(?:\s[^\]]*)?\]\](?:(?R)|.)*?\[\[/(column|col|cell)\]\]\s*)+\[\[/row\]\]\s*)+)\[\[/table\]\]\n",
        "sxi",
    );

    pub static ref PARSE_TAB_VIEW: Regex = build_regex(
        r"^\[\[(?:tabview|tabs)(\s.*?)?\]\]\s*((?:\[\[tab(\s.*?)?\]\].*?\[\[\/tab\]\]\s*)+)\[\[\/(?:tabview|tabs)\]\] *",
        "msi",
    );

    pub static ref PARSE_TABLE_OF_CONTENTS: Regex = build_regex(
        r"^(?:\n*)\[\[(f[<>])?toc( .*)?\]\](\n)*",
        "m",
    );
}

lazy_static! {
    pub static ref PARSE_MONOTYPE: Regex = build_regex(
        r"\{\{(\{*?.*\}*?)\}\}",
        "U",
    );

    pub static ref PARSE_UNDERLINE: Regex = build_regex(
        r"__([^\s](?:.*[^\s])?)__",
        "U",
    );

    pub static ref PARSE_USER: Regex = build_regex(
        r"\[\[(\*)?user ([^\]]+)\]\]",
        "i",
    );
}
