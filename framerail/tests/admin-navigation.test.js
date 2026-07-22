import assert from "node:assert/strict"
import { describe, it } from "node:test"

import {
  navigationFormValues,
  navigationUpdateValues
} from "../src/lib/admin-navigation.js"

const site = { top_bar_page: "nav:top", side_bar_page: "nav:side" }

describe("Wikidot navigation settings", () => {
  it("shows inherited site navigation for a category without overrides", () => {
    assert.deepEqual(
      navigationFormValues(
        { category_id: 12, top_bar_page: null, side_bar_page: null },
        site
      ),
      {
        categoryId: 12,
        inherit: true,
        topBarPage: "nav:top",
        sideBarPage: "nav:side"
      }
    )
  })

  it("preserves explicit category navigation values", () => {
    assert.deepEqual(
      navigationFormValues(
        {
          category_id: 13,
          top_bar_page: "nav:category-top",
          side_bar_page: "nav:category-side"
        },
        site
      ),
      {
        categoryId: 13,
        inherit: false,
        topBarPage: "nav:category-top",
        sideBarPage: "nav:category-side"
      }
    )
  })

  it("maps the inheritance control to nullable category overrides", () => {
    assert.deepEqual(
      navigationUpdateValues({
        inherit: true,
        topBarPage: "ignored",
        sideBarPage: "ignored"
      }),
      { topBarPage: null, sideBarPage: null }
    )
    assert.deepEqual(
      navigationUpdateValues({
        inherit: false,
        topBarPage: " nav:top ",
        sideBarPage: " nav:side "
      }),
      { topBarPage: "nav:top", sideBarPage: "nav:side" }
    )
  })
})
