/*
 * services/user/structs.rs
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

use crate::error::prelude::{Error, ErrorType, Result, StdResult};
use crate::models::alias::Model as AliasModel;
use crate::models::user::Model as WikijumpUserModel;
use crate::models::wikidot_user::Model as WikidotUserModel;
use crate::types::{Maybe, Reference, UserType};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::net::IpAddr;
use time::Date;

#[derive(Debug, Clone)]
pub enum User {
    Wikijump(WikijumpUserModel),
    Wikidot(WikidotUserModel),
}

impl User {
    pub fn user_id(&self) -> i64 {
        match self {
            User::Wikijump(user) => user.user_id,
            User::Wikidot(user) => i64::from(user.user_id),
        }
    }

    pub fn slug(&self) -> Option<&str> {
        match self {
            User::Wikijump(user) => Some(&user.slug),
            User::Wikidot(user) => user.slug.as_deref(),
        }
    }

    /// If this is a Wikijump user, then unwrap it and return.
    /// Otherwise, yield an error.
    pub fn unwrap_wikijump(self) -> Result<WikijumpUserModel> {
        match self {
            User::Wikijump(user) => Ok(user),
            User::Wikidot(user) => bail!(Error::new(
                "expected a wikijump user",
                ErrorType::ExpectedWikijumpUser { was_user: user },
            )),
        }
    }

    /// If this is a Wikidot user, then unwrap it and return.
    /// Otherwise, yield an error.
    pub fn unwrap_wikidot(self) -> Result<WikidotUserModel> {
        match self {
            User::Wikidot(user) => Ok(user),
            User::Wikijump(user) => bail!(Error::new(
                "expected a wikidot user",
                ErrorType::ExpectedWikidotUser { was_user: user },
            )),
        }
    }

    #[inline]
    pub fn is_wikijump(&self) -> bool {
        match self {
            User::Wikijump(_) => true,
            User::Wikidot(_) => false,
        }
    }

    #[inline]
    pub fn is_wikidot(&self) -> bool {
        match self {
            User::Wikijump(_) => false,
            User::Wikidot(_) => true,
        }
    }
}

impl From<WikijumpUserModel> for User {
    #[inline]
    fn from(user: WikijumpUserModel) -> User {
        User::Wikijump(user)
    }
}

impl From<WikidotUserModel> for User {
    #[inline]
    fn from(user: WikidotUserModel) -> User {
        User::Wikidot(user)
    }
}

// Custom serialization so we can reuse user_type for 'wikidot'
//
// For Wikijump users, user_type is 'regular', 'system', etc.
// We are using this field for user_type 'wikidot' to enable
// consumers of the serialized JSON can distinguish the different
// sets of fields.
impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;
        use time::format_description::well_known::Rfc3339;

        macro_rules! serialize_field {
            ($serializer:expr, $user:expr, $field:ident) => {
                serialize_field!($serializer, $field => $user.$field)
            };
            ($serializer:expr, $field:ident => $value:expr) => {
                $serializer.serialize_field(stringify!($field), &$value)?
            };
        }

        macro_rules! serialize_datetime {
            ($serializer:expr, $user:expr, $field:ident) => {{
                let value = $user.$field
                    .format(&Rfc3339)
                    .map_err(S::Error::custom)?;

                serialize_field!($serializer, $field => value);
            }};
        }

        match self {
            // Wikijump users are the same (it already has a user_type field)
            User::Wikijump(user) => user.serialize(serializer),

            // We add the user_type field to Wikidot users
            User::Wikidot(user) => {
                let mut object = serializer.serialize_struct("WikidotUserModel", 15)?;
                serialize_field!(object, user, user_id);
                serialize_field!(object, user_type => "wikidot");
                serialize_datetime!(object, user, created_at);
                serialize_datetime!(object, user, fetched_at);
                serialize_field!(object, user, is_deleted);
                serialize_field!(object, user, name);
                serialize_field!(object, user, slug);
                serialize_field!(object, user, avatar_s3_hash);
                serialize_field!(object, user, real_name);
                serialize_field!(object, user, gender);
                serialize_field!(object, user, birthday);
                serialize_field!(object, user, location);
                serialize_field!(object, user, biography);
                serialize_field!(object, user, website);
                serialize_field!(object, user, karma);
                serialize_field!(object, user, is_pro);
                object.end()
            }
        }
    }
}

#[derive(Debug)]
pub struct UserStub;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateUser {
    pub user_type: UserType,
    pub name: String,
    pub email: String,
    pub locales: Vec<String>,
    pub password: String,

    #[serde(default)]
    pub bypass_filter: bool,

    #[serde(default)]
    pub bypass_email_verification: bool,

    #[serde(default)]
    pub override_user_id: Option<i64>,
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateUserOutput {
    pub user_id: i64,
    pub slug: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActivateUserFromWikidot {
    pub user_id: i64,
    pub user_type: UserType,
    pub email: String,
    pub locales: Vec<String>,
    pub password: String,

    #[serde(default)]
    pub bypass_filter: bool,

    #[serde(default)]
    pub bypass_email_verification: bool,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetUser<'a> {
    pub user: Reference<'a>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetUserOutput {
    #[serde(flatten)]
    pub user: User,
    pub aliases: Vec<AliasModel>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateUser<'a> {
    pub user: Reference<'a>,

    #[serde(flatten)]
    pub body: UpdateUserBody,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct UpdateUserBody {
    pub name: Maybe<String>,
    pub email: Maybe<String>,
    pub email_verified: Maybe<bool>,
    pub password: Maybe<String>,
    pub locales: Maybe<Vec<String>>,
    pub avatar_uploaded_blob_id: Maybe<Option<String>>,
    pub real_name: Maybe<Option<String>>,
    pub gender: Maybe<Option<String>>,
    pub birthday: Maybe<Option<Date>>,
    pub location: Maybe<Option<String>>,
    pub biography: Maybe<Option<String>>,
    pub website: Maybe<Option<String>>,
    pub user_page: Maybe<Option<String>>,

    #[serde(default)]
    pub bypass_filter: bool,
}

#[test]
fn user_serialization() {
    use time::macros::{date, datetime};

    macro_rules! check {
        ($struct:expr, $json:expr $(,)?) => {{
            let object = $struct;
            let expected_json = $json.trim_start();
            let actual_json = serde_json::to_string_pretty(&object)
                .expect("Unable to serialize to JSON");

            println!("Object:\n{object:#?}\n");
            println!("Expected JSON:\n{expected_json}\n");
            println!("Actual JSON:\n{actual_json}");
            assert_eq!(
                actual_json, expected_json,
                "Actual generated JSON doesn't match expected",
            );
        }};
    }

    // Wikidot

    check!(
        User::Wikidot(WikidotUserModel {
            user_id: 1000,
            created_at: datetime!(2006-09-13 11:22:29 UTC),
            fetched_at: datetime!(2026-01-01 20:00:00 UTC),
            is_deleted: false,
            name: Some(str!("Some_user")),
            slug: Some(str!("some-user")),
            avatar_s3_hash: None,
            real_name: Some(str!("John Doe")),
            gender: Some(str!("male")),
            birthday: None,
            location: None,
            biography: Some(str!("Not a real user")),
            website: Some(str!("https://example.com")),
            karma: 0,
            is_pro: false,
        }),
        r#"
{
  "user_id": 1000,
  "user_type": "wikidot",
  "created_at": "2006-09-13T11:22:29Z",
  "fetched_at": "2026-01-01T20:00:00Z",
  "is_deleted": false,
  "name": "Some_user",
  "slug": "some-user",
  "avatar_s3_hash": null,
  "real_name": "John Doe",
  "gender": "male",
  "birthday": null,
  "location": null,
  "biography": "Not a real user",
  "website": "https://example.com",
  "karma": 0,
  "is_pro": false
}"#,
    );

    check!(
        User::Wikidot(WikidotUserModel {
            user_id: 4598089,
            created_at: datetime!(2018-09-11 22:47:35 UTC),
            fetched_at: datetime!(2026-01-01 20:01:00 UTC),
            is_deleted: false,
            name: Some(str!("aismallard")),
            slug: Some(str!("aismallard")),
            avatar_s3_hash: Some(vec![2]), // not a valid hash, just for the test
            real_name: None,
            gender: Some(str!("female")),
            birthday: None,
            location: None,
            biography: None,
            website: Some(str!("https://scpwiki.com/aismallard")),
            karma: 5,
            is_pro: false,
        }),
        r#"
{
  "user_id": 4598089,
  "user_type": "wikidot",
  "created_at": "2018-09-11T22:47:35Z",
  "fetched_at": "2026-01-01T20:01:00Z",
  "is_deleted": false,
  "name": "aismallard",
  "slug": "aismallard",
  "avatar_s3_hash": [
    2
  ],
  "real_name": null,
  "gender": "female",
  "birthday": null,
  "location": null,
  "biography": null,
  "website": "https://scpwiki.com/aismallard",
  "karma": 5,
  "is_pro": false
}"#,
    );

    check!(
        User::Wikidot(WikidotUserModel {
            user_id: 21,
            created_at: datetime!(2006-08-09 20:04:19 UTC),
            fetched_at: datetime!(2026-01-01 20:02:00 UTC),
            is_deleted: true,
            name: None,
            slug: None,
            avatar_s3_hash: Some(vec![7]),
            real_name: None,
            gender: None,
            birthday: None,
            location: None,
            biography: None,
            website: None,
            karma: 2,
            is_pro: false,
        }),
        r#"
{
  "user_id": 21,
  "user_type": "wikidot",
  "created_at": "2006-08-09T20:04:19Z",
  "fetched_at": "2026-01-01T20:02:00Z",
  "is_deleted": true,
  "name": null,
  "slug": null,
  "avatar_s3_hash": [
    7
  ],
  "real_name": null,
  "gender": null,
  "birthday": null,
  "location": null,
  "biography": null,
  "website": null,
  "karma": 2,
  "is_pro": false
}"#,
    );

    // Wikijump

    check!(
        User::Wikijump(WikijumpUserModel {
            user_id: -1,
            user_type: UserType::Regular,
            created_at: datetime!(2020-01-01 00:00:00 UTC),
            updated_at: Some(datetime!(2026-01-01 00:00:00 UTC)),
            deleted_at: None,
            name: str!("Administrator"),
            slug: str!("administrator"),
            name_changes_left: 5,
            last_name_change_added_at: datetime!(2020-01-01 00:00:00 UTC),
            last_renamed_at: None,
            email: str!("admin@wikijump.com"),
            email_verified_at: None,
            email_validation_info: None,
            email_validation_at: None,
            password: str!("!"),
            multi_factor_secret: None,
            multi_factor_recovery_codes: None,
            locales: vec![str!("en")],
            avatar_s3_hash: None,
            real_name: None,
            gender: None,
            birthday: Some(date!(1970 - 01 - 01)),
            location: Some(str!("Earth")),
            biography: Some(str!("Root platform administrator")),
            website: None,
            user_page: Some(str!("https://wikijump.com/-/admin")),
        }),
        r#"
{
  "user_id": -1,
  "user_type": "regular",
  "created_at": "2020-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z",
  "deleted_at": null,
  "name": "Administrator",
  "slug": "administrator",
  "name_changes_left": 5,
  "last_name_change_added_at": "2020-01-01T00:00:00Z",
  "last_renamed_at": null,
  "email": "admin@wikijump.com",
  "email_verified_at": null,
  "email_validation_info": null,
  "email_validation_at": null,
  "password": "!",
  "multi_factor_secret": null,
  "multi_factor_recovery_codes": null,
  "locales": [
    "en"
  ],
  "avatar_s3_hash": null,
  "real_name": null,
  "gender": null,
  "birthday": "1970-01-01",
  "location": "Earth",
  "biography": "Root platform administrator",
  "website": null,
  "user_page": "https://wikijump.com/-/admin"
}"#,
    );

    check!(
        User::Wikijump(WikijumpUserModel {
            user_id: 30000,
            user_type: UserType::Site,
            created_at: datetime!(2026-02-03 04:05:06 UTC),
            updated_at: Some(datetime!(2026-02-03 04:07:55 UTC)),
            deleted_at: None,
            name: str!("site:scp-wiki"),
            slug: str!("site:scp-wiki"),
            name_changes_left: 0,
            last_name_change_added_at: datetime!(2026-02-03 04:05:06 UTC),
            last_renamed_at: None,
            email: str!(),
            email_verified_at: None,
            email_validation_info: None,
            email_validation_at: None,
            password: str!(),
            multi_factor_secret: None,
            multi_factor_recovery_codes: None,
            locales: vec![str!("en")],
            avatar_s3_hash: None,
            real_name: None,
            gender: None,
            birthday: None,
            location: None,
            biography: None,
            website: None,
            user_page: None,
        }),
        r#"
{
  "user_id": 30000,
  "user_type": "site",
  "created_at": "2026-02-03T04:05:06Z",
  "updated_at": "2026-02-03T04:07:55Z",
  "deleted_at": null,
  "name": "site:scp-wiki",
  "slug": "site:scp-wiki",
  "name_changes_left": 0,
  "last_name_change_added_at": "2026-02-03T04:05:06Z",
  "last_renamed_at": null,
  "email": "",
  "email_verified_at": null,
  "email_validation_info": null,
  "email_validation_at": null,
  "password": "",
  "multi_factor_secret": null,
  "multi_factor_recovery_codes": null,
  "locales": [
    "en"
  ],
  "avatar_s3_hash": null,
  "real_name": null,
  "gender": null,
  "birthday": null,
  "location": null,
  "biography": null,
  "website": null,
  "user_page": null
}"#,
    );

    check!(
        User::Wikijump(WikijumpUserModel {
            user_id: 123456789,
            user_type: UserType::Bot,
            created_at: datetime!(2020-01-01 00:00:00 UTC),
            updated_at: Some(datetime!(2026-01-01 00:00:00 UTC)),
            deleted_at: Some(datetime!(2026-05-05 05:05:05 UTC)),
            name: str!("Some bot someone made"),
            slug: str!("some-bot-someone-made"),
            name_changes_left: 2,
            last_name_change_added_at: datetime!(2026-02-01 00:00:00 UTC),
            last_renamed_at: Some(datetime!(2026-03-03 03:03:03 UTC)),
            email: str!("bot@example.net"),
            email_verified_at: Some(datetime!(2026-01-01 03:15:12 UTC)),
            email_validation_info: None,
            email_validation_at: None,
            password: str!("!notarealhash_123456789"),
            multi_factor_secret: Some(str!("!notarealsecret_111")),
            multi_factor_recovery_codes: Some(vec![
                str!("!a"),
                str!("!b"),
                str!("!c"),
                str!("d"),
            ]),
            locales: vec![str!("fr"), str!("de")],
            avatar_s3_hash: Some(vec![0; 4]), // actually 64 long
            real_name: Some(str!("Bot McMuffin")),
            gender: Some(str!("sans genre")),
            birthday: Some(date!(2005 - 05 - 05)),
            location: Some(str!("France")),
            biography: None,
            website: Some(str!("https://somebot.fakesite")),
            user_page: None,
        }),
        r#"
{
  "user_id": 123456789,
  "user_type": "bot",
  "created_at": "2020-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z",
  "deleted_at": "2026-05-05T05:05:05Z",
  "name": "Some bot someone made",
  "slug": "some-bot-someone-made",
  "name_changes_left": 2,
  "last_name_change_added_at": "2026-02-01T00:00:00Z",
  "last_renamed_at": "2026-03-03T03:03:03Z",
  "email": "bot@example.net",
  "email_verified_at": "2026-01-01T03:15:12Z",
  "email_validation_info": null,
  "email_validation_at": null,
  "password": "!notarealhash_123456789",
  "multi_factor_secret": "!notarealsecret_111",
  "multi_factor_recovery_codes": [
    "!a",
    "!b",
    "!c",
    "d"
  ],
  "locales": [
    "fr",
    "de"
  ],
  "avatar_s3_hash": [
    0,
    0,
    0,
    0
  ],
  "real_name": "Bot McMuffin",
  "gender": "sans genre",
  "birthday": "2005-05-05",
  "location": "France",
  "biography": null,
  "website": "https://somebot.fakesite",
  "user_page": null
}"#,
    );
}
