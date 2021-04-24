import type { Attribute } from "./types"

// TODO: should probably make html attrs discriminate based on block name
// TODO: info getters
export const htmlAttributes: Attribute[] = [
  { name: "accept" },
  { name: "alt" },
  {
    name: "autocapitalize",
    enum: ["off", "none", "on", "sentences", "words", "characters"]
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
  { name: "decoding", enum: ["true", "false"] },
  { name: "default", boolean: true },
  { name: "dir", enum: ["ltr", "rtl"] },
  { name: "dirname" },
  { name: "disabled", boolean: true },
  // can be given without a value, like a boolean
  { name: "download" },
  // not a boolean, apparently
  // empty values aren't true either
  { name: "draggable", enum: ["true", "false"] },
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
    enum: ["none", "text", "decimal", "numeric", "tel", "search", "email", "url"]
  },
  { name: "ismap", boolean: true },
  { name: "itemprop" },
  {
    name: "kind",
    enum: ["subtitles", "captions", "descriptions", "chapters", "metadata"]
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
  { name: "preload", enum: ["none", "metadata", "auto"] },
  { name: "readonly", boolean: true },
  { name: "required", boolean: true },
  { name: "reversed", boolean: true },
  { name: "scope", enum: ["row", "col", "rowgroup", "colgroup"] },
  { name: "selected", boolean: true },
  { name: "shape", enum: ["rect", "circle", "poly", "default"] },
  { name: "size" },
  { name: "sizes" },
  // not boolean, but "true"|"false" is _required_
  { name: "spellcheck", enum: ["true", "false"] },
  { name: "src" },
  { name: "srclang" },
  { name: "srcset" },
  { name: "start" },
  { name: "step" },
  { name: "style" },
  { name: "tabindex" },
  { name: "target", enum: ["_self", "_blank", "_parent", "_top"] },
  { name: "title" },
  // this is not a boolean, and it requires "yes"|"no". seriously.
  { name: "translate", enum: ["yes", "no"] },
  // this is a union of all the possible values of "type"
  // this is really just a fallback
  {
    name: "type",
    enum: [
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
  { name: "wrap", enum: ["hard", "soft", "off"] },
  // DEPRECATED
  // probably more should go here - but it may depend on the block
  {
    name: "align",
    deprecated: true,
    enum: ["top", "middle", "bottom", "left", "right"]
  },
  { name: "background", deprecated: true },
  { name: "bgcolor", deprecated: true },
  { name: "border", deprecated: true }
]
