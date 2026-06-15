/*
 * render/handle.rs
 *
 * ftml - Library to parse Wikidot text
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

use crate::data::{KarmaLevel, PageInfo, UserInfo};
use crate::settings::WikitextSettings;
use crate::tree::{FileSource, LinkLabel, LinkLocation, Module};
use crate::url::BuildSiteUrl;
use std::borrow::Cow;
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct Handle;

impl Handle {
    pub fn render_module(&self, buffer: &mut String, module: &Module) {
        // Modules only render to HTML
        debug!("Rendering module '{}'", module.name());
        str_write!(buffer, "<p>TODO: module {}</p>", module.name());
    }

    pub fn get_page_title(&self, _site: &str, _page: &str) -> Option<String> {
        debug!("Fetching page title");

        // TODO
        Some(format!("TODO: actual title ({_site} {_page})"))
    }

    pub fn get_page_exists(&self, _site: &str, _page: &str) -> bool {
        debug!("Checking page existence");

        // For testing
        #[cfg(test)]
        if _page == "missing" {
            return false;
        }

        // TODO
        true
    }

    pub fn get_user_info<'a>(&self, name: &'a str) -> Option<UserInfo<'a>> {
        debug!("Fetching user info (name '{name}')");
        let mut info = UserInfo::dummy();
        info.user_name = cow!(name);
        info.user_profile_url = Cow::Owned(format!("/user:info/{name}"));
        Some(info)
    }

    pub fn get_file_link<'a>(
        &self,
        source: &FileSource<'a>,
        info: &PageInfo,
        settings: &WikitextSettings,
    ) -> Option<Cow<'a, str>> {
        let (site, page, file): (&str, &str, &str) = match source {
            FileSource::Url(url) => return Some(Cow::clone(url)),
            FileSource::File1 { .. }
            | FileSource::File2 { .. }
            | FileSource::File3 { .. }
                if !settings.allow_local_paths =>
            {
                warn!("Specified path file source when local paths are disabled");
                return None;
            }
            FileSource::File1 { file } => (&info.site, &info.page, file),
            FileSource::File2 { page, file } => (&info.site, page, file),
            FileSource::File3 { site, page, file } => (site, page, file),
        };

        // TODO: emit url
        Some(Cow::Owned(format!(
            "https://{site}.wjfiles.com/local--files/{page}/{file}",
        )))
    }

    pub fn get_link_label<F>(
        &self,
        site: &str,
        link: &LinkLocation,
        label: &LinkLabel,
        f: F,
    ) where
        F: FnOnce(&str),
    {
        let page_title;
        let label_text = match label {
            LinkLabel::Text(text) | LinkLabel::Slug(text) => text,
            LinkLabel::Url => match link {
                LinkLocation::Url(url) => url.as_ref(),
                LinkLocation::Page(_) => {
                    panic!("Requested a URL link label for a page");
                }
            },
            LinkLabel::Page => match link {
                LinkLocation::Page(page_ref) => {
                    let (site, page, _) = page_ref.fields_or(site);
                    page_title = match self.get_page_title(site, page) {
                        Some(title) => title,
                        None => page_ref.to_string(),
                    };

                    &page_title
                }
                LinkLocation::Url(_) => {
                    panic!("Requested a page title link label for a URL");
                }
            },
        };

        f(label_text);
    }

    pub fn get_karma_style(&self, karma: KarmaLevel) -> &'static str {
        // TODO replace these with inline data image URIs
        match karma {
            KarmaLevel::Zero => {
                "background-image: url(https://www.wikidot.com/userkarma.php?u=8976177)"
            }
            KarmaLevel::One => {
                "background-image: url(https://www.wikidot.com/userkarma.php?u=172570)"
            }
            KarmaLevel::Two => {
                "background-image: url(https://www.wikidot.com/userkarma.php?u=172952)"
            }
            KarmaLevel::Three => {
                "background-image: url(https://www.wikidot.com/userkarma.php?u=172904)"
            }
            KarmaLevel::Four => {
                "background-image: url(https://www.wikidot.com/userkarma.php?u=6040770)"
            }
            KarmaLevel::Five => {
                "background-image: url(https://www.wikidot.com/userkarma.php?u=4598089)"
            }
        }
    }

    pub fn get_message(&self, language: &str, message: &str) -> &'static str {
        debug!("Fetching message (language {language}, key {message})");

        let _ = language;

        // TODO
        match message {
            "button-copy-clipboard" => "Copy to Clipboard",
            "collapsible-open" => "+ open block",
            "collapsible-hide" => "- hide block",
            "table-of-contents" => "Table of Contents",
            "footnote" => "Footnote",
            "footnote-block-title" => "Footnotes",
            "bibliography-reference" => "Reference",
            "bibliography-block-title" => "Bibliography",
            "bibliography-cite-not-found" => "Bibliography item not found",
            "image-context-bad" => "No images in this context",
            "user-missing-pre" => "",
            "user-missing-post" => " does not match any existing user name",
            _ => {
                error!("Unknown message requested (key {message})");
                "?"
            }
        }
    }

    pub fn post_html(&self, info: &PageInfo, html: &str) -> String {
        debug!("Submitting HTML to create iframe-able snippet");

        let _ = info;
        let _ = html;

        // TODO
        str!("https://example.com/")
    }

    pub fn post_code(&self, index: NonZeroUsize, code: &str) {
        debug!("Submitting code snippet (index {})", index.get());

        let _ = index;
        let _ = code;

        // TODO
    }
}

impl BuildSiteUrl for Handle {
    fn build_url(&self, site: &str, path: &str, extra: Option<&str>) -> String {
        // TODO make this a parser setting
        // get url of wikijump instance here

        // TODO
        let extra = extra.unwrap_or("");
        format!("https://{site}.wikijump.com/{path}{extra}")
    }
}
