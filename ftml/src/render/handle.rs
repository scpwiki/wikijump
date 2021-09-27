/*
 * render/handle.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use crate::log::prelude::*;
use crate::tree::{ImageSource, LinkLabel, LinkLocation, Module};
use crate::url::BuildSiteUrl;
use crate::{PageInfo, UserInfo};
use std::borrow::Cow;
use std::num::NonZeroUsize;
use strum_macros::IntoStaticStr;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct Handle;

impl Handle {
    pub fn render_module(
        &self,
        log: &Logger,
        buffer: &mut String,
        module: &Module,
        mode: ModuleRenderMode,
    ) {
        info!(
            log,
            "Rendering module";
            "module" => module.name(),
            "mode" => mode.name(),
        );

        match mode {
            ModuleRenderMode::Html => {
                str_write!(buffer, "<p>TODO: module {}</p>", module.name());
            }
            ModuleRenderMode::Text => {
                str_write!(buffer, "TODO: module {}", module.name());
            }
        }
    }

    pub fn get_page_title(&self, log: &Logger, link: &LinkLocation) -> String {
        info!(log, "Fetching page title"; "link" => link);

        // TODO
        format!("TODO: actual title ({:?})", link)
    }

    pub fn get_user_info<'a>(&self, log: &Logger, name: &'a str) -> Option<UserInfo<'a>> {
        info!(log, "Fetching user info"; "name" => name);

        let mut info = UserInfo::dummy();
        info.user_name = cow!(name);
        info.user_profile_url = Cow::Owned(format!("/user:info/{}", name));
        Some(info)
    }

    pub fn get_image_link<'a>(
        &self,
        log: &Logger,
        info: &PageInfo,
        source: &ImageSource<'a>,
    ) -> Cow<'a, str> {
        info!(log, "Getting file link for image");

        let (site, page, file): (&str, &str, &str) = match source {
            ImageSource::Url(url) => return Cow::clone(url),
            ImageSource::File1 { file } => (&info.site, &info.page, file),
            ImageSource::File2 { page, file } => (&info.site, page, file),
            ImageSource::File3 { site, page, file } => (site, page, file),
        };

        // TODO
        Cow::Owned(format!(
            "https://{}.wjfiles.com/local--files/{}/{}",
            site, page, file,
        ))
    }

    pub fn get_link_label<F>(
        &self,
        log: &Logger,
        link: &LinkLocation,
        label: &LinkLabel,
        f: F,
    ) where
        F: FnOnce(&str),
    {
        let page_title;
        let label_text = match *label {
            LinkLabel::Text(ref text) => text,
            LinkLabel::Url(Some(ref text)) => text,
            LinkLabel::Url(None) => match link {
                LinkLocation::Url(url) => url,
                LinkLocation::Page(page_ref) => page_ref.page(),
            },
            LinkLabel::Page => {
                page_title = self.get_page_title(log, link);
                &page_title
            }
        };

        f(label_text);
    }

    pub fn get_message(
        &self,
        log: &Logger,
        language: &str,
        message: &str,
    ) -> &'static str {
        info!(
            log,
            "Fetching message";
            "language" => language,
            "message" => message,
        );

        let _ = language;

        // TODO
        match message {
            "button-copy-clipboard" => "Copy to Clipboard",
            "collapsible-open" => "+ open block",
            "collapsible-hide" => "- hide block",
            "table-of-contents" => "Table of Contents",
            "footnote" => "Footnote",
            "footnote-block-title" => "Footnotes",
            _ => {
                info!(
                    log,
                    "Unknown message requested";
                    "message" => message,
                );

                "?"
            }
        }
    }

    pub fn post_html(&self, log: &Logger, info: &PageInfo, html: &str) -> String {
        info!(log, "Submitting HTML to create iframe-able snippet");

        let _ = info;
        let _ = html;

        // TODO
        str!("https://example.com/")
    }

    pub fn post_code(&self, log: &Logger, index: NonZeroUsize, code: &str) {
        info!(
            log,
            "Submitting code snippet";
            "index" => index.get(),
            "code" => code,
        );

        let _ = index;
        let _ = code;

        // TODO
    }
}

impl BuildSiteUrl for Handle {
    fn build_url(&self, site: &str, path: &str) -> String {
        // TODO make this a parser setting
        // get url of wikijump instance here

        let path = {
            let mut path = str!(path);
            normalize(&mut path);
            path
        };

        // TODO
        format!("https://{}.wikijump.com/{}", site, path)
    }
}

#[derive(
    IntoStaticStr, Serialize, Deserialize, Debug, Hash, Copy, Clone, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleRenderMode {
    Html,
    Text,
}

impl ModuleRenderMode {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
