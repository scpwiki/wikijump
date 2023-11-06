## Limits

Wikijump is intended to be used for very large sites which contain numerous pages, revisions, and other content. As such, it is appropriate to describe what the absolute limits of its storage ability are.
This does not describe the performance capabilities of a Wikijump instance, but rather the maximal values the IDs for these objects can take.

To understand the limits for Wikidot-imported items, see [Compatibility IDs](compatibility-ids.md).

| Item                             | Upper limit               | Comment |
|----------------------------------|---------------------------|---------|
| Users (total)                    | 9223372036854775807       |         |
| Users (new)                      | 9223372036844775807       | Any new user accounts registered on Wikijump. |
| Users (compatibility)            | 10000000                  | Any user accounts imported from Wikidot. |
| Sites                            | 9223372036854775807       |         |
| Pages (total)                    | 9223372036854775807       |         |
| Pages (new)                      | 9223372033854775807       | Any new pages created on Wikijump. |
| Pages (compatibility)            | 3000000000                | Any pages imported from Wikidot. Realistically this value is 2147483647, which is the maximum ID for pages on Wikidot. |
| Pages (per site)                 | None                      | There is no limit specifically on the number of pages per site. The total number of pages is limited, see above. |
| Revisions (total)                | 9223372036854775807       |         |
| Revisions (new)                  | 9223372033854775807       | Any new page revisions created on Wikijump. |
| Revisions (compatibility)        | 3000000000                | Any page revisions imported from Wikidot. Realistically this value is 2147483647, which is the maximum ID for pages on Wikidot. |
| Revisions (per page)             | 2147483647                |         |
| Page Categories (total)          | 9223372036854775807       |         |
| Page Categories (per site)       | None                      | There is no limit specifically on the number of categories per site. The total number of categories is limited, see above. |
| Forum Groups                     | 9223372036854775807       | Value not exposed in Wikidot, assigned incrementally. |
| Forum Categories (total)         | 9223372036854775807       |         |
| Forum Categories (new)           | 9223372036845775807       |         |
| Forum Categories (compatibility) | 9000000                   |         |
| Forum Threads (total)            | 9223372036854775807       |         |
| Forum Threads (new)              | 9223372036824775807       |         |
| Forum Threads (compatbility)     | 30000000                  |         |
| Forum Posts (total)              | 9223372036854775807       |         |
| Forum Posts (new)                | 9223372036847775807       |         |
| Forum Posts (compatibility)      | 7000000                   |         |

Also note that, to avoid bugs confusing `0` with `NULL`, all counters for sequential numeric IDs start at least 1, in effect lowering each effective upper limit by 1.
