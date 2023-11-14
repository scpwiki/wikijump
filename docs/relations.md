## Relations

Relations (formerly "interactions") are a data structure in wide use within Wikijump, due to its flexible ability to represent a wide array of data items without the need for dedicated database tables, and thus, largely duplicated CRUD sevice logic.

bluesoul originally created this concept in Laravel, and established the basic foundation of what the data type represents. See the code for the [migration](https://github.com/scpwiki/wikijump/blob/legacy-php/web/database/migrations/2021_07_30_231009_create_interactions_table.php) and [model](https://github.com/scpwiki/wikijump/blob/legacy-php/web/app/Models/Interaction.php).

It has relation type and a pair of IDs which are mapped to each other. For instance, we can represent site membership as a `member` type between a site ID (`dest`) and a user ID (`from`), or a user block as a `block` type between a user ID (`dest`, the blocking user) and another user ID (`from`, the blocked user). In this case, because the two types of IDs are the same, a relation could be made in the other direction, as you'd expect for user blocks, whereas for memberships, there is an asymmetry due to the type difference.

Then, each relation can have optional additional data attached as JSON. Some relations have no further information (e.g. user blocks), whereas a relation like a site ban can have information such as the reason and ban expiration date (if any). This provides flexibility for the different needs of each relation type.

There is a macro system which implements the relation methods in `RelationService` for you, to avoid writing out a lot of boilerplate. For simple relations, just invoking `impl_relation!` alone is sufficient, but for some, you may wish to implement the `create_*` method yourself. For instance, with user blocks, applying a block has side effects, such as terminating any contact relationships, or a relation may only be applied in certain situations, for instance site users are not permitted to apply to sites that they are banned from.

This system also asserts that the metadata type matches that which is expected. Because this is Rust, we value type safety, so the macro also ensures that each relation type has a corresponding metadata type (which can be as simple as `()`, null, for simple relations). This helps tame the wildness of JSON, though do remember at the end of the day it is backed by such, so choose reasonable serialization representations and respect backwards compatibility (or implement a migration) if you need to make changes.

When you begin to see in this paradigm, you realize many data relationships we care about in the wiki world can be represented as relations rather than as dedicated tables. For instance:

* Page attribution
* Page parents
* User blocks
* Site blocks (aka bans)
* Watching or starring pages
* Bot user ownership
* Site applications
* and more

When adding a new data type or relation to Wikijump, think for a second whether this could actually be a relation with associated metadata. In more times than you think, it can be done, and using the interaction system can save you a lot of implementation time in not needing to write boilerplate code or CRUD interfaces, and can improve long-term maintainability by not introducing a new service which has little unique logic.

There are some cases where, despite fitting into the relation model, having separate tables can make sense. For instance:

* Page connections / links &mdash; This is also associated with broken page links and external URLs, which both use strings, which relations do not support. Rather than having only one of these tables as a relation and the rest as separate tables, they are all non-relation tables.
* Page votes &mdash; This is an obvious relation candidate (page / user with vote value as the metadata), but because `ScorerService` uses direct database queries rather than fetching raw records for more efficient score calculation, storing integers in a JSON column is unhelpful for processing large groups of votes.
