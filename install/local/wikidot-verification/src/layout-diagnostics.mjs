export const DEFAULT_COMPUTED_STYLE_WHITELIST = [
  "display",
  "visibility",
  "opacity",
  "position",
  "overflow",
  "overflow-x",
  "overflow-y",
  "z-index",
  "font-size",
  "line-height",
  "width",
  "height",
  "transform",
  "background-image",
  "object-fit",
  "max-width",
  "max-height",
  "--logo",
  "--header-title",
  "--header-subtitle",
];

export const DEFAULT_SCP9506_DESCRIPTORS = [
  {name: "header", selector: "#header"},
  {name: "top_bar", selector: "#top-bar"},
  {name: "side_bar", selector: "#side-bar"},
  {name: "main_content", selector: "#main-content"},
  {name: "page_title", selector: "#page-title"},
  {name: "ios_cache_issue_notification", selector: ".ios-cache-issue-notification"},
  {name: "page_content", selector: "#page-content"},
  {name: "rate_widget", selector: ".page-rate-widget-box"},
  {name: "nfsi_name", selector: ".name"},
  {name: "nfsi_navigation", selector: ".navigation"},
  {name: "nfsi_start", selector: ".start"},
  {name: "article_blocks", selector: ".article"},
  {name: "article_images", selector: ".article img"},
  {name: "collapsible_blocks", selector: "details.collapsible-block"},
  {name: "scp9506_local_file_images", selector: "#page-content img[src*=\"/local--files/scp-9506/\"]"},
  {name: "interwiki_frames", selector: "iframe[src*=\"interwikiFrame.html\"]"},
  {name: "style_frames", selector: "iframe[src*=\"styleFrame.html\"]"},
];

export function parseViewport(value) {
  const match = /^(\d+)x(\d+)$/iu.exec(String(value ?? "").trim());
  if (!match) {
    throw new Error("viewport must use WIDTHxHEIGHT");
  }
  const width = Number.parseInt(match[1], 10);
  const height = Number.parseInt(match[2], 10);
  if (width <= 0 || height <= 0) {
    throw new Error("viewport width and height must be positive integers");
  }
  return {width, height};
}

export async function collectElementDiagnostics(
  page,
  descriptors = DEFAULT_SCP9506_DESCRIPTORS,
  computedStyles = DEFAULT_COMPUTED_STYLE_WHITELIST,
  {maxInstancesPerDescriptor = 20} = {},
) {
  return await page.evaluate(
    ({descriptors, computedStyles, maxInstancesPerDescriptor}) => {
      function rectObject(element) {
        const rect = element.getBoundingClientRect();
        return {
          x: round(rect.x),
          y: round(rect.y),
          width: round(rect.width),
          height: round(rect.height),
          top: round(rect.top),
          right: round(rect.right),
          bottom: round(rect.bottom),
          left: round(rect.left),
        };
      }

      function round(value) {
        return Math.round(Number(value) * 100) / 100;
      }

      function compactText(value) {
        return String(value ?? "").replace(/\s+/gu, " ").trim().slice(0, 240);
      }

      function rendered(rect, styles) {
        return (
          rect.width > 0 &&
          rect.height > 0 &&
          styles.display !== "none" &&
          styles.visibility !== "hidden" &&
          styles.opacity !== "0"
        );
      }

      return descriptors.map((descriptor) => {
        const nodes = Array.from(document.querySelectorAll(descriptor.selector));
        const instances = nodes.slice(0, maxInstancesPerDescriptor).map((element, index) => {
          const computed = getComputedStyle(element);
          const styles = {};
          for (const property of computedStyles) {
            styles[property] = computed.getPropertyValue(property);
          }
          const rect = rectObject(element);
          const instance = {
            index,
            tag: element.tagName,
            rect,
            styles,
            rendered: rendered(rect, styles),
            text: compactText(element.innerText ?? element.textContent),
          };
          if (element instanceof HTMLImageElement) {
            instance.current_src = element.currentSrc || element.src || null;
            instance.natural_width = element.naturalWidth;
            instance.natural_height = element.naturalHeight;
            instance.complete = element.complete;
          }
          if (element instanceof HTMLIFrameElement) {
            instance.src = element.src || element.getAttribute("src") || null;
          }
          return instance;
        });
        return {
          name: descriptor.name,
          selector: descriptor.selector,
          found_count: nodes.length,
          truncated: nodes.length > instances.length,
          instances,
        };
      });
    },
    {descriptors, computedStyles, maxInstancesPerDescriptor},
  );
}

export async function collectDocumentMetrics(page) {
  return await page.evaluate(() => ({
    title: document.title,
    url: location.href,
    client_width: document.documentElement.clientWidth,
    client_height: document.documentElement.clientHeight,
    scroll_width: document.documentElement.scrollWidth,
    scroll_height: document.documentElement.scrollHeight,
    body_scroll_width: document.body?.scrollWidth ?? null,
    body_scroll_height: document.body?.scrollHeight ?? null,
  }));
}

export async function installLayoutShiftObserver(page) {
  await page.addInitScript(() => {
    window.__wikijumpLayoutShifts = [];
    if (typeof PerformanceObserver !== "function") return;
    try {
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.hadRecentInput) continue;
          window.__wikijumpLayoutShifts.push({
            value: entry.value,
            startTime: entry.startTime,
            sources: (entry.sources ?? []).map((source) => ({
              node: source.node?.tagName ?? null,
              previousRect: rect(source.previousRect),
              currentRect: rect(source.currentRect),
            })),
          });
        }
      });
      observer.observe({type: "layout-shift", buffered: true});
    } catch {
      window.__wikijumpLayoutShiftsUnsupported = true;
    }

    function rect(value) {
      if (!value) return null;
      return {
        x: value.x,
        y: value.y,
        width: value.width,
        height: value.height,
      };
    }
  });
}

export async function collectLayoutShifts(page) {
  return await page.evaluate(() => ({
    supported: !window.__wikijumpLayoutShiftsUnsupported,
    entries: window.__wikijumpLayoutShifts ?? [],
    cls: (window.__wikijumpLayoutShifts ?? []).reduce((sum, entry) => sum + Number(entry.value ?? 0), 0),
  }));
}

export function evaluateLayoutInvariants(diagnostics) {
  const invariants = [];
  const anomalies = [];

  add("page_status_200", diagnostics.page?.status === 200, `status is ${diagnostics.page?.status ?? "unknown"}`);
  add("no_failed_requests", (diagnostics.page?.failed_requests ?? []).length === 0, `${(diagnostics.page?.failed_requests ?? []).length} failed requests`);
  add("no_console_errors", (diagnostics.page?.console_errors ?? []).length === 0, `${(diagnostics.page?.console_errors ?? []).length} console errors`);
  add("main_content_nonzero", renderedElement(diagnostics, "main_content"), "#main-content has a rendered box");
  add("page_content_nonzero", renderedElement(diagnostics, "page_content"), "#page-content has a rendered box");

  const rate = firstInstance(diagnostics, "rate_widget");
  add(
    "rate_widget_snapshot_value",
    Boolean(rate?.rendered && /\+371/u.test(rate.text ?? "")),
    `.page-rate-widget-box text is ${JSON.stringify(rate?.text ?? "")}`,
  );

  const localImages = allInstances(diagnostics, "scp9506_local_file_images").filter((item) => item.rendered);
  add(
    "visible_scp9506_image_present",
    localImages.some((item) => Number(item.natural_width ?? 0) > 0 && Number(item.natural_height ?? 0) > 0),
    `${localImages.length} visible scp-9506 local-file images`,
  );
  add(
    "visible_scp9506_images_have_natural_dimensions",
    localImages.every((item) => Number(item.natural_width ?? 0) > 0 && Number(item.natural_height ?? 0) > 0),
    "all visible scp-9506 local-file images report natural dimensions",
  );

  const iosOverlayVisible = allInstances(diagnostics, "ios_cache_issue_notification").some((item) => item.rendered);
  add("ios_cache_notification_hidden", !iosOverlayVisible, "iOS cache notification has no visible rendered overlay");

  const duplicateLabel = allInstances(diagnostics, "collapsible_blocks").some((item) =>
    /(More From This Author){2}|‡ Licensing \/ Citation‡/u.test(item.text ?? ""),
  );
  add("collapsible_labels_not_duplicated", !duplicateLabel, "known collapsible labels are not duplicated");

  const documentMetrics = diagnostics.page?.document ?? {};
  const horizontalOverflow =
    Number(documentMetrics.scroll_width ?? 0) > Number(documentMetrics.client_width ?? 0) + 1;
  record(
    "horizontal_overflow_recorded",
    !horizontalOverflow,
    `scroll width ${documentMetrics.scroll_width ?? "unknown"}, client width ${documentMetrics.client_width ?? "unknown"}`,
  );

  const failed = invariants.filter((item) => item.status === "fail").length;
  const recorded = invariants.filter((item) => item.status === "recorded").length;
  return {
    summary: {
      status: failed === 0 ? "pass" : "fail",
      total: invariants.length,
      failed,
      recorded,
    },
    invariants,
    anomalies,
  };

  function add(id, passed, detail) {
    const row = {id, status: passed ? "pass" : "fail", detail};
    invariants.push(row);
    if (!passed) anomalies.push(row);
  }

  function record(id, passed, detail) {
    const row = {id, status: passed ? "pass" : "recorded", detail};
    invariants.push(row);
    if (!passed) anomalies.push(row);
  }
}

export function buildDiagnosticsRecord({
  fixtureId,
  url,
  viewport,
  status,
  finalUrl,
  failedRequests = [],
  consoleErrors = [],
  document,
  elements,
  layoutShifts,
}) {
  const diagnostics = {
    schema: "wikijump_local_lab.layout_diagnostics.v1",
    generated_at: new Date().toISOString(),
    fixture_id: fixtureId,
    url,
    viewport,
    page: {
      status,
      final_url: finalUrl,
      failed_requests: failedRequests,
      console_errors: consoleErrors,
      document,
    },
    elements,
    layout_shifts: layoutShifts,
  };
  return {
    ...diagnostics,
    verdict: evaluateLayoutInvariants(diagnostics),
  };
}

function renderedElement(diagnostics, name) {
  return allInstances(diagnostics, name).some((item) => item.rendered);
}

function firstInstance(diagnostics, name) {
  return allInstances(diagnostics, name)[0] ?? null;
}

function allInstances(diagnostics, name) {
  return (diagnostics.elements ?? []).find((item) => item.name === name)?.instances ?? [];
}
