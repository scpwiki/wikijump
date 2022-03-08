## Compatibility IDs

Backwards compatibility with Wikidot is an important goal of the Wikijump project. In order to allow imported data from Wikidot to be usable in Wikijump, the project utilizes "compatibility IDs" in order to ensure that any ID valid on Wikidot is also valid on Wikijump (if the relevant data is imported).

This is important because Wikidot exposes IDs in several places, such as forum URLs, which we need to preserve in some way in order for all those legacy links to remain valid.

The approach used here is to set the starting ID value to be higher than Wikidot in production will ever conceivably reach (at least before Wikijump is deployed). We did this by finding a recent ID value for that kind of object, and increasing that increases its most significant digit by at least one value. For the most populous objects (pages and revisions), the produced value is higher than Wikidot could ever achieve, since they use only 32-bit integers for IDs.

```
- Page           -- 3000000000 (sample       1331370625)
- Revision       -- 3000000000 (sample       1388179085)
- Forum Post     --    7000000 (sample          5174477)
- Forum Thread   --   30000000 (sample         14447612)
- Forum Category --    9000000 (sample          7412040)
- User           --   10000000 (sample          7840760)
```

And as a point of comparison, these are the maximum values for 32-bit and 64-bit signed integers:

```
- i32 max        --            (max          2147483647)
- i64 max        --            (max 9223372036854775807)
```

This change was implemented in [WJ-964](https://scuttle.atlassian.net/browse/WJ-964).
