import type { Attribute } from "./types"

// TODO: should probably make html attrs discriminate based on block name
// TODO: info getters
export const htmlAttributes: Attribute[] = [
  { name: "accept" },
  { name: "alt" },
  {
    name: "autocapitalize",
    values: ["off", "none", "on", "sentences", "words", "characters"]
  },
  { name: "autoplay", boolean: true },
  // not accessible from HTML, apparently?
  // { name: "buffered", values: ["true", "false"] },
  { name: "checked", boolean: true },
  { name: "cite" },
  { name: "class" },
  { name: "cols" },
  { name: "colspan" },
  // for the purposes of FTML, this is a boolean attribute
  // however, in HTML it's actually enumerated
  { name: "contenteditable", boolean: true },
  { name: "controls", boolean: true },
  { name: "coords" },
  { name: "datetime" },
  { name: "decoding", values: ["true", "false"] },
  { name: "default", boolean: true },
  { name: "dir", values: ["ltr", "rtl"] },
  { name: "dirname" },
  { name: "disabled", boolean: true },
  // can be given without a value, like a boolean
  { name: "download" },
  // not a boolean, apparently
  // empty values aren't true either
  { name: "draggable", values: ["true", "false"] },
  { name: "for" },
  { name: "form" },
  { name: "headers" },
  { name: "height" },
  { name: "hidden", boolean: true },
  { name: "high" },
  { name: "href" },
  { name: "hreflang" },
  { name: "id" },
  {
    name: "inputmode",
    values: ["none", "text", "decimal", "numeric", "tel", "search", "email", "url"]
  },
  { name: "ismap", boolean: true },
  { name: "itemprop" },
  {
    name: "kind",
    values: ["subtitles", "captions", "descriptions", "chapters", "metadata"]
  },
  { name: "label" },
  { name: "lang" },
  { name: "list" },
  { name: "loop", boolean: true },
  { name: "low" },
  { name: "max" },
  { name: "maxlength" },
  { name: "minlength" },
  { name: "min" },
  { name: "multiple", boolean: true },
  { name: "muted", boolean: true },
  { name: "name" },
  { name: "optimum" },
  { name: "pattern" },
  { name: "placeholder" },
  { name: "poster" },
  // empty string is "auto"
  { name: "preload", values: ["none", "metadata", "auto"] },
  { name: "readonly", boolean: true },
  { name: "required", boolean: true },
  { name: "reversed", boolean: true },
  { name: "scope", values: ["row", "col", "rowgroup", "colgroup"] },
  { name: "selected", boolean: true },
  { name: "shape", values: ["rect", "circle", "poly", "default"] },
  { name: "size" },
  { name: "sizes" },
  // not boolean, but "true"|"false" is _required_
  { name: "spellcheck", values: ["true", "false"] },
  { name: "src" },
  { name: "srclang" },
  { name: "srcset" },
  { name: "start" },
  { name: "step" },
  { name: "style" },
  { name: "tabindex" },
  { name: "target", values: ["_self", "_blank", "_parent", "_top"] },
  { name: "title" },
  // this is not a boolean, and it requires "yes"|"no". seriously.
  { name: "translate", values: ["yes", "no"] },
  // this is a union of all the possible values of "type"
  // this is really just a fallback
  {
    name: "type",
    values: [
      "submit",
      "reset",
      "button",
      "checkbox",
      "color",
      "date",
      "datetime-local",
      "email",
      "file",
      "hidden",
      "image",
      "month",
      "number",
      "password",
      "radio",
      "range",
      "reset",
      "search",
      "submit",
      "tel",
      "text",
      "time",
      "url",
      "week"
    ]
  },
  { name: "usemap" },
  { name: "width" },
  { name: "wrap", values: ["hard", "soft", "off"] },
  // DEPRECATED
  // probably more should go here - but it may depend on the block
  {
    name: "align",
    deprecated: true,
    values: ["top", "middle", "bottom", "left", "right"]
  },
  { name: "background", deprecated: true },
  { name: "bgcolor", deprecated: true },
  { name: "border", deprecated: true }
]
