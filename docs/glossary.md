## Wikijump Glossary

There are a number of concepts within Wikidot, which can take on a number of different names in casual usage.
To standardize their usage during development and within the API, this document will describe the standard name, Wikidot's name (if different), and any other aliases.

Definitions are not intended to be exhaustive, but rather quickly summarize what it is for recognition. If further details are needed, add a link to a more detailed resource.

| Official Term  | Wikidot Term | Other Terms | Definition |
|----------------|--------------|-------------|------------|
| Site           |              | Wiki        | A subdomain of Wikijump which has an independent wiki, with its own pages, staff, and settings. |
| Page           |              | Article     | The main data unit of Wikidot, containing wikitext and metadata, and existing at exactly one particular URL at a time. |
| Slug           | UNIX name    | Fullname    | The "file" portion of a page's URL, always in [Wikidot normal form](https://scuttle.atlassian.net/wiki/spaces/WD/pages/541655041/Wikidot+Normal+Form). |
| Normalization  | `toUnixName`, `unixify` |  | The process of converting a string into Wikidot normal form. |
| Page category  |              |             | A particular namespace a page may exist in. Denoted by the part of a slug before the `:`. If a colon is absent, then the page is in category `_default`. |
| Wikitext       | Source       | Wikidot code, ftml | The textual representation of the contents of a page. |
| Vote           |              |             | An individual vote cast on a page, along with its timestamp and user information. |
| Score          | Rating       |             | The calculated result of all the votes presently active on a page, per the site's chosen scoring algorithm. |
| Rating         | Rating       |             | Wikidot's simple scoring algorithm, equivalent to the sum of `upvotes - downvotes`. |
| File           |              | Attachment, upload | A blob attached to a page, with an associated filename and MIME type. |
| Revision       |              | Edit        | A particular set of changes on a page at a point in time. It has an associated timestamp, user, and revision message attached. |
| User           |              | Account     | A user account on Wikijump, with its own username, avatar, and credentials. |
| Member         |              |             | A user, specifically in the context of its membership on a particular Wikijump site. |
| Guest          |              |             | A user (or an anonymous individual who is not logged in) who is accessing a Wikijump site. This is the inverse of a member. |
| Forum category |              | Forum       | A namespace that a collection of forum threads may exist in. They may be visible or not, and have an associated name. |
| Forum thread   |              | Thread      | A thread of conversation within a particular category, containing zero or more posts. They are either created by a user or by Wikijump. |
| Forum post     |              | Post        | An individual message sent within a thread. It can be a child or reply to another post, or top-level. These are only created by users. |
