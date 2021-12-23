import { FluentResource } from "@fluent/bundle"
import { sleep } from "@wikijump/util"
import { get } from "svelte/store"
import { describe, expect, it, spyOn } from "vitest"
import * as lib from "../src/index"
import type { FluentImportMap } from "../src/locales"

const fixture = `
foo = bar
  .baz = qux

asdf = { $placeholder }

footer-foo = bar
`

const en_de_map: FluentImportMap = {
  "en": () => Promise.resolve(fixture),
  "de": () => Promise.resolve(fixture)
}

const de_map: FluentImportMap = {
  "de": () => Promise.resolve("no = nein")
}

const fr_map: FluentImportMap = {
  "fr-CA": () => Promise.resolve("oui = yes")
}

const component = new lib.FluentComponent("resource", en_de_map)

describe("@wikijump/fluent", () => {
  describe("component", () => {
    it("has", () => {
      expect(component.has("en")).toBe(true)
      expect(component.has("de")).toBe(true)
      expect(component.has("fr")).toBe(false)
    })

    it("which", () => {
      const langs = ["en", "de", "fr"]
      expect(component.which(langs)).toBe("en")
      expect(component.which(langs.reverse())).toBe("de")
    })

    it("loads resources", async () => {
      const resource = await component.load("en")
      expect(resource).toBeInstanceOf(FluentResource)
    })

    it("caches resources", async () => {
      const resource = await component.load("de")
      expect(resource).toBe(await component.load("de"))
    })
  })

  describe("locale", () => {
    const locale = new lib.Locale("en", "de")

    it("parses basic selectors", () => {
      const [id, attribute] = lib.Locale.parseSelector("foo")
      expect(id).toBe("foo")
      expect(attribute).toBe(null)
    })

    it("parses attribute selectors", () => {
      const [id, attribute] = lib.Locale.parseSelector("foo.baz")
      expect(id).toBe("foo")
      expect(attribute).toBe("baz")
    })

    it("adds a resource", async () => {
      const resource = new FluentResource("wiki = jump")
      await locale.add(resource)
      expect(locale.format("wiki")).toBe("jump")
    })

    it("adds a component", async () => {
      await locale.add(component)
      expect(locale.format("foo")).toBe("bar")
      expect(locale.format("foo.baz")).toBe("qux")
    })

    it("adds a component using a fallback", async () => {
      const component = new lib.FluentComponent("resource", de_map)
      await locale.add(component)
      expect(locale.format("no")).toBe("nein")
    })

    it("handles unsupported locale in components", async () => {
      const component = new lib.FluentComponent("resource", fr_map)
      await locale.add(component)
      expect(locale.format("oui")).not.toBe("yes")
    })

    it("gets a pattern with fallback", async () => {
      await locale.add(component)
      // @ts-ignore - private method
      expect(locale.getPattern("bad", "foo")).toBe("bar")
    })

    it("loads a component only once", async () => {
      // the second promise should not result in another call to `add`
      const spied = spyOn(locale, "add")
      await locale.load("emails")
      await locale.load("emails")
      expect(spied).toHaveBeenCalledTimes(1)
    })

    it("makeComponentFormatter", async () => {
      await locale.load("base")
      const store = locale.makeComponentFormatter("footer")
      let formatter = get(store)
      // breaks if we change this string, unfortunately
      expect(formatter("bad")).toBe("Loading...")
      await locale.load("footer")
      // waiting until the end of the current event loop
      await sleep(0)
      formatter = get(store)
      // just returns itself if the message can't be found
      expect(formatter("bad")).toBe("bad")
    })

    it("makeComponentFormatter shorthands", () => {
      const store = locale.makeComponentFormatter("footer")
      const formatter = get(store)
      expect(formatter("#-foo")).toBe("bar")
    })

    it("has", async () => {
      await locale.add(component)
      expect(locale.has("foo")).toBe(true)
      expect(locale.has("foo.baz")).toBe(true)
      expect(locale.has("bar")).toBe(false)
    })

    it("formats", async () => {
      await locale.add(component)
      expect(locale.format("foo")).toBe("bar")
      expect(locale.format("foo.baz")).toBe("qux")
      expect(locale.format("asdf", { placeholder: "foobar" })).toBe("foobar")
    })

    // don't need to thoroughly test this, it's just Intl.NumberFormat
    it("number", () => {
      const str = locale.number(123456789)
      expect(str).toBe("123,456,789")
    })

    it("unit", () => {
      expect(locale.unit(1, "millimeter")).toBe("1 mm")
      expect(locale.unit(1, "mile-scandinavian-per-fahrenheit")).toBe("1 smi/°F")
    })
  })
})
