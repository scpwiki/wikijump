import assert from "node:assert/strict"
import { describe, it } from "node:test"

import { ratingFormValues } from "../src/lib/admin-rating.js"

/** @type {NonNullable<Parameters<typeof ratingFormValues>[1]>} */
const defaultCategory = {
  category_id: 1,
  slug: "_default",
  rating_enabled: true,
  rating_permission: "members",
  rating_visibility: "anonymous",
  rating_type: "stars"
}

describe("Wikidot category rating settings", () => {
  it("shows the _default category settings while a category inherits", () => {
    assert.deepEqual(
      ratingFormValues(
        {
          category_id: 12,
          slug: "article",
          rating_enabled: null,
          rating_permission: null,
          rating_visibility: null,
          rating_type: null
        },
        defaultCategory
      ),
      {
        categoryId: 12,
        inherit: true,
        enabled: true,
        permission: "members",
        visibility: "anonymous",
        ratingType: "stars"
      }
    )
  })

  it("preserves explicit category overrides", () => {
    assert.deepEqual(
      ratingFormValues(
        {
          category_id: 13,
          slug: "article",
          rating_enabled: false,
          rating_permission: "registered",
          rating_visibility: "visible",
          rating_type: "plus"
        },
        defaultCategory
      ),
      {
        categoryId: 13,
        inherit: false,
        enabled: false,
        permission: "registered",
        visibility: "visible",
        ratingType: "plus"
      }
    )
  })

  it("uses the evidenced Wikidot defaults when no stored baseline exists", () => {
    assert.deepEqual(
      ratingFormValues(
        {
          category_id: 1,
          slug: "_default",
          rating_enabled: null,
          rating_permission: null,
          rating_visibility: null,
          rating_type: null
        },
        undefined
      ),
      {
        categoryId: 1,
        inherit: false,
        enabled: true,
        permission: "registered",
        visibility: "visible",
        ratingType: "plus_minus"
      }
    )
  })
})
