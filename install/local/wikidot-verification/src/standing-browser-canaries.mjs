import { createHash } from "node:crypto";

export const STANDING_BROWSER_CANARY_SCHEMA =
  "wikijump_local_lab.standing_browser_canaries.v1";
export const DEFAULT_VIEWPORT = Object.freeze({ width: 1366, height: 900 });
export const DEFAULT_SETTLE_MS = 1_000;
export const DEFAULT_TIMEOUT_MS = 900_000;
export const COMMON_GEOMETRY_SELECTORS = Object.freeze([
  "#main-content",
  "#page-content",
  "#header",
  "#side-bar",
  "#header h1 a",
]);
export const REQUIRED_THEME_FAMILIES = Object.freeze([
  "sigma",
  "basalt",
  "flopstyle-y2k",
  "black-highlighter-calibri",
]);

const common = Object.freeze({
  geometry_selectors: COMMON_GEOMETRY_SELECTORS,
  presence_probes: Object.freeze([
    Object.freeze({
      id: "header_logo_element",
      selector: "#header h1 a span",
      minimum_count: 1,
      require_rendered: true,
    }),
    Object.freeze({
      id: "header_subtitle_element",
      selector: "#header h2 span",
      minimum_count: 1,
      require_rendered: false,
    }),
  ]),
});

const HEADER_PSEUDO_PROPERTIES = Object.freeze([
  "content",
  "background-image",
  "color",
  "display",
  "visibility",
  "opacity",
  "width",
  "height",
  "max-width",
  "max-height",
  "overflow",
  "overflow-x",
  "overflow-y",
  "position",
  "z-index",
  "top",
  "right",
  "bottom",
  "left",
  "margin-top",
  "margin-bottom",
  "padding-top",
  "padding-bottom",
  "line-height",
  "font-size",
  "font-weight",
  "font-family",
  "letter-spacing",
  "white-space",
  "text-overflow",
  "clip",
  "clip-path",
  "transform",
]);

function canary({
  slug,
  themeFamily,
  geometrySelectors = [],
  presenceProbes = [],
  firstPaintCustomProperties = {},
}) {
  return Object.freeze({
    slug,
    theme_family: themeFamily,
    geometry_selectors: Object.freeze([
      ...common.geometry_selectors,
      ...geometrySelectors,
    ]),
    presence_probes: Object.freeze([
      ...common.presence_probes,
      ...presenceProbes,
    ]),
    first_paint_custom_properties: Object.freeze(firstPaintCustomProperties),
  });
}

// SCP-9506 exercises the Basalt header and page-supplied navigation, while
// the remaining rows cover every production theme family represented by the
// standing stack. A selector count is deliberately insufficient for the
// Basalt pseudo-elements: their visible painted area is checked separately.
export const STANDING_BROWSER_CANARIES = Object.freeze([
  canary({
    slug: "scp-9506",
    themeFamily: "basalt",
    geometrySelectors: ["#page-content .navigation"],
    presenceProbes: [
      Object.freeze({
        id: "header_logo",
        selector: "#header h1 a",
        pseudo: "::before",
        minimum_count: 1,
        require_rendered: true,
        comparison_properties: HEADER_PSEUDO_PROPERTIES,
        visibility_container_selector: "#header",
        pseudo_layout: Object.freeze({
          require_background_image: true,
          allow_css_box_fallback: true,
          minimum_visible_area_ratio: 0.95,
        }),
      }),
      Object.freeze({
        id: "header_title",
        selector: "#header h1 a span",
        pseudo: "::before",
        minimum_count: 1,
        require_rendered: true,
        comparison_properties: HEADER_PSEUDO_PROPERTIES,
        visibility_container_selector: "#header",
        pseudo_layout: Object.freeze({
          require_generated_content: true,
          require_descendant_text: true,
          minimum_visible_area_ratio: 0.95,
        }),
      }),
      Object.freeze({
        id: "header_subtitle",
        selector: "#header h1 a span",
        pseudo: "::after",
        minimum_count: 1,
        require_rendered: true,
        comparison_properties: HEADER_PSEUDO_PROPERTIES,
        visibility_container_selector: "#header",
        pseudo_layout: Object.freeze({
          require_generated_content: true,
          require_descendant_text: true,
          minimum_visible_area_ratio: 0.95,
        }),
      }),
      Object.freeze({
        id: "navigation_tab_bar",
        selector: "#page-content .navigation",
        minimum_count: 1,
        require_rendered: true,
      }),
      Object.freeze({
        id: "navigation_tab_links",
        selector: "#page-content .navigation a",
        minimum_count: 6,
        require_rendered: true,
      }),
    ],
    firstPaintCustomProperties: {
      "--logo": Object.freeze({
        operator: "contains",
        value: "/local--files/scp-9506/NFSI.png",
      }),
      "--header-logo": Object.freeze({
        operator: "contains",
        value: "/local--files/scp-9506/NFSI.png",
      }),
      "--header-title": Object.freeze({ operator: "eq", value: '"NFSI"' }),
      "--header-subtitle": Object.freeze({
        operator: "eq",
        value: '"National Fog Safety Initiative"',
      }),
    },
  }),
  canary({ slug: "scp-744", themeFamily: "flopstyle-y2k" }),
  canary({ slug: "scp-2117", themeFamily: "sigma" }),
  canary({ slug: "scp-5516", themeFamily: "black-highlighter-calibri" }),
  canary({ slug: "scp-8980", themeFamily: "basalt" }),
  canary({
    slug: "theme:basalt",
    themeFamily: "basalt",
    geometrySelectors: [".yui-navset"],
    presenceProbes: [
      Object.freeze({
        id: "tab_navset",
        selector: ".yui-navset",
        minimum_count: 1,
        require_rendered: true,
      }),
    ],
  }),
]);

export function canarySlug(url) {
  const pathname = new URL(url).pathname.replace(/^\/+|\/+$/gu, "");
  return decodeURIComponent(pathname);
}

export function canaryForUrl(url) {
  const slug = canarySlug(url);
  return (
    STANDING_BROWSER_CANARIES.find((candidate) => candidate.slug === slug) ??
    null
  );
}

export function localUrlForCanary(canary, origin) {
  return new URL(`/${encodeURI(canary.slug)}`, origin).href;
}

export function liveUrlForCanary(canary, origin) {
  return new URL(`/${encodeURI(canary.slug)}`, origin).href;
}

export function defaultCanaryPairs({ localOrigin, liveOrigin }) {
  return STANDING_BROWSER_CANARIES.map((canary) =>
    Object.freeze({
      canary_slug: canary.slug,
      theme_family: canary.theme_family,
      local_url: localUrlForCanary(canary, localOrigin),
      live_url: liveUrlForCanary(canary, liveOrigin),
    }),
  );
}

export function assertThemeFamilyCoverage(
  canaries = STANDING_BROWSER_CANARIES,
) {
  const present = new Set(canaries.map((canary) => canary.theme_family));
  const missing = REQUIRED_THEME_FAMILIES.filter(
    (family) => !present.has(family),
  );
  if (missing.length > 0) {
    throw new Error(
      `standing canaries are missing production theme families: ${missing.join(", ")}`,
    );
  }
  return Object.freeze([...present].sort());
}

export function firstPaintPropertyNames(canary) {
  return Object.keys(canary?.first_paint_custom_properties ?? {}).sort();
}

export function standingBrowserCanaryContract(
  canaries = STANDING_BROWSER_CANARIES,
  requiredThemeFamilies = REQUIRED_THEME_FAMILIES,
) {
  return {
    schema: STANDING_BROWSER_CANARY_SCHEMA,
    required_theme_families: [...requiredThemeFamilies],
    canaries,
  };
}

export function standingBrowserCanaryContractSha256(
  canaries = STANDING_BROWSER_CANARIES,
  requiredThemeFamilies = REQUIRED_THEME_FAMILIES,
) {
  return createHash("sha256")
    .update(
      JSON.stringify(
        standingBrowserCanaryContract(canaries, requiredThemeFamilies),
      ),
    )
    .digest("hex");
}

assertThemeFamilyCoverage();
