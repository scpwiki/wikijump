/*
 * services/text_block/mime.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use paste::paste;

/// Convenience macro to make the MIME string constants.
///
/// This is so we don't have to repeat `pub const` and `charset=utf-8` each time.
/// It is assumed since all Rust strings must be UTF-8.
macro_rules! mime {
    ($name:ident, $mime:expr) => {
        paste! {
            pub const [<MIME_ $name>]: &str = concat!($mime, "; charset=utf-8");
        }
    };
}

// Thanks to https://www.digipres.org/formats/mime-types/ for many of these.
mime!(ACTIONSCRIPT, "text/actionscript");
mime!(APL, "text/apl");
mime!(ASM, "text/x-asm");
mime!(C, "text/x-c");
mime!(CLOJURE, "text/x-clojure");
mime!(COBOL, "text/x-cobol");
mime!(COFFEESCRIPT, "text/x-coffeescript");
mime!(COMMON_LISP, "text/x-common-lisp");
mime!(CPP, "text/x-c++src");
mime!(CPP_HEADER, "text/x-c++hdr");
mime!(CSHARP, "text/x-csharp");
mime!(CSS, "text/css");
mime!(CSV, "text/csv");
mime!(D, "text/x-d");
mime!(DIFF, "text/x-diff");
mime!(DTD, "text/x-dtd");
mime!(EIFFEL, "text/x-eiffel");
mime!(ERLANG, "text/x-erlang");
mime!(EXAMPLE, "text/example"); // lol
mime!(FORTRAN, "text/x-fortran");
mime!(FTML, "text/x-ftml");
mime!(FSHARP, "text/x-fsharp");
mime!(GO, "text/x-go");
mime!(GROOVY, "text/x-groovy");
mime!(HAML, "text/x-haml");
mime!(HASKELL, "text/x-haskell");
mime!(HTML, "text/html");
mime!(INI, "text/x-ini");
mime!(JAVA, "text/x-java-source");
mime!(JAVASCRIPT, "text/javascript");
mime!(JSON, "application/json");
mime!(LEX, "text/x-lex");
mime!(LUA, "text/x-lua");
mime!(MARKDOWN, "text/markdown");
mime!(MATHML, "text/mathml");
mime!(OCAML, "text/x-ocaml");
mime!(PASCAL, "text/x-pascal");
mime!(PERL, "text/x-perl");
mime!(PHP, "text/x-php");
mime!(PROLOG, "text/x-prolog");
mime!(PYTHON, "text/x-python");
mime!(RST, "text/x-rst");
mime!(RUBY, "text/x-ruby");
mime!(RUST, "text/rust");
mime!(SCALA, "text/x-scala");
mime!(SCHEME, "text/x-scheme");
mime!(SED, "text/x-sed");
mime!(SQL, "application/sql");
mime!(TEX, "text/x-tex");
mime!(TEXT, "text/plain");
mime!(TSV, "text/tab-separated-values");
mime!(TYPESCRIPT, "text/x-typescript");
mime!(XML, "application/xml");
mime!(YACC, "text/x-yacc");
mime!(YAML, "text/x-yaml");

/// Gets the MIME type for a given `[[code]]` language specification.
///
/// This supports everything Wikidot does and more.
pub fn mime_for_language<S: AsRef<str>>(language: &Option<S>) -> &'static str {
    const MAPPING: [(&str, &str); 75] = [
        // These are at the top since they're the most common.
        ("css", MIME_CSS),
        ("html", MIME_HTML),
        ("js", MIME_JAVASCRIPT),
        ("javascript", MIME_JAVASCRIPT),
        ("ftml", MIME_FTML),
        ("wikidot", MIME_FTML),
        ("wikijump", MIME_FTML),
        // Everything else
        ("actionscript", MIME_ACTIONSCRIPT),
        ("apl", MIME_APL),
        ("asm", MIME_ASM),
        ("c", MIME_C),
        ("c#", MIME_CSHARP),
        ("c++ header", MIME_CPP_HEADER),
        ("c++", MIME_CPP),
        ("clojure", MIME_CLOJURE),
        ("cobol", MIME_COBOL),
        ("coffeescript", MIME_COFFEESCRIPT),
        ("common lisp", MIME_COMMON_LISP),
        ("cpp", MIME_CPP),
        ("cxx", MIME_CPP),
        ("hpp", MIME_CPP_HEADER),
        ("hxx", MIME_CPP_HEADER),
        ("cpp header", MIME_CPP_HEADER),
        ("cxx header", MIME_CPP_HEADER),
        ("cs", MIME_CSHARP),
        ("csharp", MIME_CSHARP),
        ("csv", MIME_CSV),
        ("d", MIME_D),
        ("diff", MIME_DIFF),
        ("dtd", MIME_DTD),
        ("eiffel", MIME_EIFFEL),
        ("erlang", MIME_ERLANG),
        ("example", MIME_EXAMPLE),
        ("f#", MIME_FSHARP),
        ("fortran", MIME_FORTRAN),
        ("fsharp", MIME_FSHARP),
        ("go", MIME_GO),
        ("go", MIME_GO),
        ("groovy", MIME_GROOVY),
        ("haml", MIME_HAML),
        ("haskell", MIME_HASKELL),
        ("ini", MIME_INI),
        ("java", MIME_JAVA),
        ("json", MIME_JSON),
        ("lex", MIME_LEX),
        ("lua", MIME_LUA),
        ("md", MIME_MARKDOWN),
        ("markdown", MIME_MARKDOWN),
        ("mathml", MIME_MATHML),
        ("ocaml", MIME_OCAML),
        ("pascal", MIME_PASCAL),
        ("pl", MIME_PERL),
        ("perl", MIME_PERL),
        ("php", MIME_PHP),
        ("prolog", MIME_PROLOG),
        ("py", MIME_PYTHON),
        ("python", MIME_PYTHON),
        ("rst", MIME_RST),
        ("rb", MIME_RUBY),
        ("ruby", MIME_RUBY),
        ("rs", MIME_RUST),
        ("rust", MIME_RUST),
        ("scala", MIME_SCALA),
        ("scheme", MIME_SCHEME),
        ("sed", MIME_SED),
        ("sql", MIME_SQL),
        ("tex", MIME_TEX),
        ("latex", MIME_TEX),
        ("tsv", MIME_TSV),
        ("typescript", MIME_TYPESCRIPT),
        ("ts", MIME_TYPESCRIPT),
        ("xml", MIME_XML),
        ("yacc", MIME_YACC),
        ("yaml", MIME_YAML),
        ("yml", MIME_YAML),
    ];

    let language = match language {
        Some(language) => language.as_ref(),
        None => return MIME_TEXT,
    };

    debug!("Getting MIME type for language code '{language}'");

    for (lang, mime) in MAPPING {
        if language.eq_ignore_ascii_case(lang) {
            return mime;
        }
    }

    warn!("Unknown MIME type for language '{language}', returning text");
    MIME_TEXT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_for_language_maps_known_aliases_and_defaults_to_text() {
        assert_eq!(mime_for_language(&Some("css")), MIME_CSS);
        assert_eq!(mime_for_language(&Some("JavaScript")), MIME_JAVASCRIPT);
        assert_eq!(mime_for_language(&Some("wikidot")), MIME_FTML);
        assert_eq!(mime_for_language(&Some("rs")), MIME_RUST);
        assert_eq!(mime_for_language(&Some("unknown-language")), MIME_TEXT);
        assert_eq!(mime_for_language::<&str>(&None), MIME_TEXT);
    }
}
