/*
 * database/seeder/data.rs
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

use crate::error::prelude::*;
use crate::types::{License, UserType};
use ftml::layout::Layout;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use time::Date;

#[derive(Debug)]
pub struct SeedData {
    pub users: Vec<User>,
    pub sites: Vec<Site>,
    pub pages: HashMap<String, Vec<Page>>,
    pub files: HashMap<String, HashMap<String, Vec<File>>>,
    pub filters: Vec<Filter>,
    pub roles: Vec<Role>,
}

impl SeedData {
    pub fn load(directory: &Path) -> Result<Self> {
        let make_error =
            || Error::new("failed to load seed data", ErrorType::DatabaseSeeder);

        let mut path: PathBuf = directory.join("filename");

        // Load user data
        let users: Vec<User> =
            Self::load_json(&mut path, "users").or_raise(make_error)?;

        // Load site data
        let sites: Vec<Site> =
            Self::load_json(&mut path, "sites").or_raise(make_error)?;

        // Load page data
        let mut site_pages: HashMap<String, Vec<Page>> =
            Self::load_json(&mut path, "pages").or_raise(make_error)?;

        for (site, pages) in &mut site_pages {
            // Verify that the site exists
            assert!(
                sites.iter().any(|s| &s.slug == site),
                "No site with slug {site}",
            );

            // Fetch wikitext from file
            for page in pages {
                page.wikitext = Self::load_wikitext(&mut path, &page.wikitext_filename)
                    .or_raise(make_error)?;
            }
        }

        // Load file data
        let files: HashMap<String, HashMap<String, Vec<File>>> =
            Self::load_json(&mut path, "files").or_raise(make_error)?;

        // Load filter data
        let filters: Vec<Filter> =
            Self::load_json(&mut path, "filters").or_raise(make_error)?;

        // Load roles template
        let roles: Vec<Role> =
            Self::load_json(&mut path, "roles").or_raise(make_error)?;

        // Build and return
        Ok(SeedData {
            users,
            sites,
            pages: site_pages,
            files,
            filters,
            roles,
        })
    }

    fn load_json<T>(path: &mut PathBuf, filename: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let make_error = || Error::new("failed to load JSON", ErrorType::DatabaseSeeder);

        path.set_file_name(filename);
        path.set_extension("json");
        debug!("Loading JSON from {}", path.display());

        let mut file = fs::File::open(path).or_raise(make_error)?;
        let data = serde_json::from_reader(&mut file).or_raise(make_error)?;
        Ok(data)
    }

    fn load_wikitext(path: &mut PathBuf, filename: &Path) -> Result<String> {
        let make_error =
            || Error::new("failed to load wikitext", ErrorType::DatabaseSeeder);

        path.set_file_name(filename);
        path.set_extension("ftml");
        debug!("Loading wikitext from {}", path.display());

        let wikitext = fs::read_to_string(path).or_raise(make_error)?;
        Ok(wikitext)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub email: String,

    #[serde(rename = "type")]
    pub user_type: UserType,
    pub password: Option<String>,
    pub locales: Vec<String>,
    pub real_name: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<Date>,
    pub location: Option<String>,
    pub biography: Option<String>,
    pub user_page: Option<String>,
    pub aliases: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Site {
    pub slug: String,

    #[serde(default)]
    pub aliases: Vec<String>,

    #[serde(default)]
    pub domains: Vec<SiteDomain>,

    #[serde(default)]
    pub preferred_domain: Option<String>,
    pub name: String,
    pub tagline: String,
    pub description: String,

    #[serde(default)]
    pub default_page: Option<String>,
    pub layout: Option<Layout>,
    pub license: License,
    pub locale: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum SiteDomain {
    DomainOnly(String),
    WithRedirect {
        domain: String,

        #[serde(rename = "www-redirect", default = "default_true")]
        www_redirect: bool,
    },
}

impl SiteDomain {
    pub fn domain(&self) -> &str {
        match self {
            SiteDomain::DomainOnly(domain) => domain,
            SiteDomain::WithRedirect { domain, .. } => domain,
        }
    }

    pub fn into_fields(self) -> (String, bool) {
        match self {
            SiteDomain::DomainOnly(domain) => (domain, true),
            SiteDomain::WithRedirect {
                domain,
                www_redirect,
            } => (domain, www_redirect),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Page {
    pub slug: String,
    pub title: String,

    #[serde(default)]
    pub alt_title: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub created_by: Option<i64>,

    #[serde(skip)]
    pub wikitext: String,

    #[serde(rename = "wikitext")]
    pub wikitext_filename: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Filter {
    pub regex: String,
    pub description: String,
    pub case_sensitive: bool,

    #[serde(rename = "site")]
    pub site_slug: Option<String>,

    #[serde(default)]
    pub user: bool,

    #[serde(default)]
    pub email: bool,

    #[serde(default)]
    pub page: bool,

    #[serde(default)]
    pub file: bool,

    #[serde(default)]
    pub forum: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct File {
    pub name: String,
    pub path: PathBuf,

    #[serde(default)]
    pub overwrite: Option<PathBuf>,

    #[serde(default)]
    pub deleted: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Role {
    pub name: String,
    pub description: String,
    pub is_virtual: bool,
    pub parent_role: Option<String>,
    pub permissions: Vec<String>,
}

#[inline]
const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::SeedData;
    use std::path::Path;

    #[test]
    fn loads_reserved_scp_wiki_mirror_pages() {
        let seeder_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeder");
        let seed = SeedData::load(&seeder_path).expect("seed data should load");
        let pages = seed.pages.get("scp-wiki").expect("scp-wiki pages");

        let scp_3922 = pages
            .iter()
            .find(|page| page.slug == "scp-3922")
            .expect("scp-3922 seed page");
        assert_eq!(scp_3922.title, "SCP-3922");
        assert_eq!(scp_3922.tags, ["scp"]);
        assert!(scp_3922.wikitext.contains("**Item #:** SCP-3922"));

        let scp_9506 = pages
            .iter()
            .find(|page| page.slug == "scp-9506")
            .expect("scp-9506 seed page");
        assert_eq!(scp_9506.title, "National Fog Safety Initiative");
        assert_eq!(scp_9506.tags, ["scp"]);
        assert!(scp_9506.wikitext.contains("National Fog Safety Initiative"));
        assert!(scp_9506.wikitext.contains("local--files/scp-9506/NFSI.png"));

        let files = seed.files.get("scp-wiki").expect("scp-wiki files");
        let scp_3922_files = files.get("scp-3922").expect("scp-3922 files");
        assert!(scp_3922_files.iter().any(|file| file.name == "theend.jpg"
            && file.path == Path::new("scp-3922--theend.jpg")));

        let basalt_files = files.get("theme:basalt").expect("theme:basalt files");
        assert!(
            basalt_files
                .iter()
                .any(|file| file.name == "basalt_scp_logo-for_lightmode.svg")
        );
        assert!(
            basalt_files
                .iter()
                .any(|file| file.name == "O5_DARKLOGO.png")
        );

        let theme_basalt = pages
            .iter()
            .find(|page| page.slug == "theme:basalt")
            .expect("theme:basalt seed page");
        assert_eq!(theme_basalt.tags, ["theme"]);

        let component_betterfootnotes = pages
            .iter()
            .find(|page| page.slug == "component:betterfootnotes")
            .expect("component:betterfootnotes seed page");
        assert_eq!(component_betterfootnotes.tags, ["component"]);

        let page_index = |slug: &str| {
            pages
                .iter()
                .position(|page| page.slug == slug)
                .unwrap_or_else(|| panic!("missing seeded page {slug}"))
        };
        assert!(
            page_index("component:image-block-base")
                < page_index("component:image-block")
        );
        assert!(page_index("component:interwiki-style") < page_index("theme:basalt"));
        assert!(page_index("component:betterfootnotes") < page_index("theme:basalt"));
        assert!(page_index("component:acs-animation") < page_index("theme:basalt"));
        assert!(
            page_index("component:license-box-backend")
                < page_index("component:license-box")
        );
        assert!(
            page_index("component:license-box-end") < page_index("component:license-box")
        );
    }
}
