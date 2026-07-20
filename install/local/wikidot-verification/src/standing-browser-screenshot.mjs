import fs from "node:fs/promises";

export async function capturePng(page, destination, { fullPage = false } = {}) {
  let client;
  try {
    client = await page.context().newCDPSession(page);
    const options = {
      format: "png",
      fromSurface: true,
      captureBeyondViewport: fullPage,
    };
    if (fullPage) {
      const metrics = await client.send("Page.getLayoutMetrics");
      const size = metrics.cssContentSize ?? metrics.contentSize;
      if (!size || !(size.width > 0 && size.height > 0)) {
        throw new Error(
          "Page.getLayoutMetrics returned no positive content size",
        );
      }
      options.clip = {
        x: 0,
        y: 0,
        width: size.width,
        height: size.height,
        scale: 1,
      };
    }
    const { data } = await client.send("Page.captureScreenshot", options);
    if (typeof data !== "string" || data.length === 0) {
      throw new Error("Page.captureScreenshot returned no PNG data");
    }
    const bytes = Buffer.from(data, "base64");
    await fs.writeFile(destination, bytes, { mode: 0o600 });
    return { path: destination, bytes: bytes.length, full_page: fullPage };
  } finally {
    await client?.detach().catch(() => {});
  }
}
