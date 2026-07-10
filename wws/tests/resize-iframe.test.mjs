import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import test from "node:test";
import vm from "node:vm";

const HANDLER_SOURCE = new URL("../src/handler/misc.rs", import.meta.url);

function embeddedResizeScript() {
  const source = readFileSync(HANDLER_SOURCE, "utf8");
  const html = source.match(
    /const RESIZE_IFRAME_HTML: &str = r#"([\s\S]*?)"#;/,
  )?.[1];
  assert.ok(html, "RESIZE_IFRAME_HTML must remain extractable");

  const script = html.match(
    /<script type="text\/javascript">([\s\S]*?)<\/script>/,
  )?.[1];
  assert.ok(script, "resize-iframe.html must contain its executable script");
  return script;
}

function runResizeScript(hash, sources) {
  const elements = sources.map((src) => ({
    height: null,
    getAttribute(name) {
      assert.equal(name, "src");
      return src;
    },
  }));
  const selectors = [];

  function collection(items) {
    return {
      filter(callback) {
        return collection(
          items.filter((item, index) => callback.call(item, index, item)),
        );
      },
      height(value) {
        for (const item of items) item.height = value;
        return this;
      },
    };
  }

  const context = {
    location: {
      hash,
      toString() {
        return this.hash;
      },
    },
    parent: {
      parent: {
        $j(selector) {
          selectors.push(selector);
          return collection(elements);
        },
      },
    },
  };

  vm.runInNewContext(embeddedResizeScript(), context, {
    filename: "resize-iframe.html",
  });
  return {elements, selectors};
}

test("resizes only the iframe with the generated decimal block id", () => {
  const {elements, selectors} = runResizeScript("#321/1", [
    "/page/html/1",
    "/page/html/11",
    "/page/html/2",
  ]);

  assert.deepEqual(selectors, ["iframe.html-block-iframe"]);
  assert.deepEqual(
    elements.map(({height}) => height),
    ["321px", null, null],
  );
});

test("preserves legacy Wikidot iframe hash ids", () => {
  const id = "5d5a0384a922dd96ac0db81d715a5bf348d43c57";
  const {elements} = runResizeScript(`#42/${id}`, [
    `/local--html/start/${id}`,
    "/local--html/start/other",
  ]);

  assert.deepEqual(
    elements.map(({height}) => height),
    ["42px", null],
  );
});

test("selector payloads cannot alter the constant iframe selector", () => {
  const raw = runResizeScript('#42/1"],#secret', ["/page/html/1"]);
  assert.deepEqual(raw.selectors, []);
  assert.equal(raw.elements[0].height, null);

  const encoded = runResizeScript("#42/%22%5D%2C%23secret", [
    "/page/html/1",
  ]);
  assert.deepEqual(encoded.selectors, ["iframe.html-block-iframe"]);
  assert.equal(encoded.elements[0].height, null);
});

test("accepts bounded heights and rejects values above the limit", () => {
  for (const height of [0, 99999, 100000]) {
    const result = runResizeScript(`#${height}/1`, ["/page/html/1"]);
    assert.deepEqual(result.selectors, ["iframe.html-block-iframe"]);
    assert.equal(result.elements[0].height, `${height}px`);
  }

  const aboveLimit = runResizeScript("#100001/1", ["/page/html/1"]);
  assert.deepEqual(aboveLimit.selectors, []);
  assert.equal(aboveLimit.elements[0].height, null);
});
