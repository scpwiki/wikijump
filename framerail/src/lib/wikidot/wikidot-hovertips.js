import { isJapaneseWikidotLocale } from "./wikidot-locale.js"

/**
 * Wikidot page-option and footer hovertips.
 *
 * Every Wikidot page ends its body with two hidden containers,
 * `#page-options-bottom-tips` and `#page-options-bottom-2-tips`, holding
 * one `<div id="<control>-hovertip">` per control. On DOM ready Wikidot
 * calls `OZONE.dialog.hovertip.dominit("html-body", {delay: 700, valign:
 * "center"})`, which moves each tip whose control exists into
 * `#odialog-hovertips` and shows it while the pointer is over the control.
 * The texts are the raw server HTML of scp-wiki and scp-jp `scp-173`
 * (captured 2026-09-29 and 2026-10-11); the behavior is a port of
 * `OZONE.dialog.hovertip` from the retained Wikidot `init.combined.js`.
 */

/** @typedef {readonly (readonly [string, string, readonly string[]])[]} WikidotHovertipTexts */

/**
 * Text nodes the standard server template leaves around each tip text:
 * `[before, after]`. dominit() moves them into `div.content` unchanged and
 * a hidden tip's innerText exposes them, so they are part of the observed
 * DOM.
 */
const PAGE_TIP = Object.freeze(["\n        ", "    "])
const PARENT_TIP = Object.freeze(["  \n        ", "    "])
const FOOTER_TIP = Object.freeze(["\n            ", "        "])
const LAST_FOOTER_TIP = Object.freeze(["\n            ", "          \n        "])

const ENGLISH_TIPS = Object.freeze({
  first: Object.freeze([
    ["edit-button", "Click here to edit contents of this page.", PAGE_TIP]
  ]),
  second: Object.freeze([
    [
      "edit-sections-button",
      "Click here to toggle editing of individual sections of the page (if possible).         Watch headings for an &quot;edit&quot; link when available.",
      PAGE_TIP
    ],
    [
      "edit-append-button",
      "Append content without editing the whole page source.",
      PAGE_TIP
    ],
    ["history-button", "Check out how this page has evolved in the past.", PAGE_TIP],
    [
      "discuss-button",
      "If you want to discuss contents of this page - this is the easiest way to do it.",
      PAGE_TIP
    ],
    ["files-button", "View and manage file attachments for this page.", PAGE_TIP],
    ["site-tools-button", "A few useful tools to manage this Site.", PAGE_TIP],
    ["backlinks-button", "See pages that link to and include this page.", PAGE_TIP],
    [
      "rename-move-button",
      "Change the name (also URL address, possibly the category) of the page.",
      PAGE_TIP
    ],
    ["view-source-button", "View wiki source for this page without editing.", PAGE_TIP],
    [
      "parent-page-button",
      "View/set parent page (used for creating breadcrumbs and structured layout).",
      PARENT_TIP
    ],
    [
      "abuse-report-button",
      "Notify administrators if there is objectionable content in this page.",
      FOOTER_TIP
    ],
    [
      "bug-report-button",
      "Something does not work as expected? Find out what you can do.",
      FOOTER_TIP
    ],
    [
      "wikidot-help-button",
      "General Wikidot.com documentation and help section.",
      FOOTER_TIP
    ],
    [
      "wikidot-tos-button",
      "Wikidot.com Terms of Service - what you can, what you should not etc.",
      FOOTER_TIP
    ],
    ["wikidot-privacy-button", "Wikidot.com Privacy Policy.", LAST_FOOTER_TIP]
  ])
})

const JAPANESE_TIPS = Object.freeze({
  first: Object.freeze([
    ["edit-button", "ページを編集するにはこのボタンをクリックしてください。", PAGE_TIP]
  ]),
  second: Object.freeze([
    [
      "edit-sections-button",
      "セクションごとの編集を切り替えるにはこのボタンをクリックしてください(ページにセクションが設定されている必要があります）。有効になった場合はセクションに&quot;編集&quot;ボタンが設置されます。",
      PAGE_TIP
    ],
    [
      "edit-append-button",
      "ページのソース全体を編集せずに、コンテンツを追加します。",
      PAGE_TIP
    ],
    [
      "history-button",
      "このページが過去にどのように変化したかを調べることができます。",
      PAGE_TIP
    ],
    [
      "discuss-button",
      "このページについて話をしたいときは、これを使うのが一番簡単な方法です。",
      PAGE_TIP
    ],
    [
      "files-button",
      "このページに添付されたファイルの閲覧や管理を行うことができます。",
      PAGE_TIP
    ],
    ["site-tools-button", "サイトの管理についての便利なツール。", PAGE_TIP],
    [
      "backlinks-button",
      "このページへリンクしている、またはこのページをincludeしているページを表示します。",
      PAGE_TIP
    ],
    [
      "rename-move-button",
      "このページの名前(それに伴いURLやページのカテゴリも)を変更します。",
      PAGE_TIP
    ],
    ["view-source-button", "編集せずにこのページのソースコードを閲覧します。", PAGE_TIP],
    [
      "parent-page-button",
      "親ページを設定/閲覧できます(パンくずリストの作成やサイトの構造化に用いられます)",
      PARENT_TIP
    ],
    ["abuse-report-button", "管理者にページの違反を通知する。", FOOTER_TIP],
    [
      "bug-report-button",
      "何か思い通りにいかないことがありますか? 何ができるか調べましょう。",
      FOOTER_TIP
    ],
    [
      "wikidot-help-button",
      "Wikidot.comのシステム概要とヘルプセクションです。",
      FOOTER_TIP
    ],
    [
      "wikidot-tos-button",
      "Wikidot 利用規約 ― 何ができるか、何をすべきでないか etc.",
      FOOTER_TIP
    ],
    ["wikidot-privacy-button", "Wikidot.com プライバシーポリシー", LAST_FOOTER_TIP]
  ])
})

/** @param {string | null | undefined} locale */
export const wikidotHovertipTexts = (locale) =>
  isJapaneseWikidotLocale(locale) ? JAPANESE_TIPS : ENGLISH_TIPS

/** @param {WikidotHovertipTexts} tips */
const tipsHtml = (tips) =>
  tips
    .map(
      ([control, text, [before, after]]) =>
        `<div id="${control}-hovertip">${before}${text}${after}</div>`
    )
    .join("")

/**
 * The two hidden tip containers Wikidot emits at the end of every page
 * body. Texts are trusted constants already in their HTML form.
 *
 * @param {string | null | undefined} locale
 */
export const buildWikidotHovertipContainersHtml = (locale) => {
  const { first, second } = wikidotHovertipTexts(locale)
  return (
    `<div id="page-options-bottom-tips" style="display: none;">${tipsHtml(first)}</div>` +
    `<div id="page-options-bottom-2-tips" style="display: none;">${tipsHtml(second)}</div>`
  )
}

// OZONE.dialog.hovertip keeps tips 20px from the pointer and the viewport edge.
const POINTER_GAP = 20

/**
 * Port of `OZONE.dialog.hovertip._mousemove` for the `valign: "center"`
 * option that `dominit("html-body", ...)` uses: center the tip under the
 * pointer, flip it above the pointer near the viewport bottom, and keep it
 * inside the viewport horizontally.
 *
 * @param {{ pageY: number; clientX: number; clientY: number }} pointer
 * @param {{ width: number; height: number }} tip
 * @param {{ width: number; height: number }} viewport
 * @returns {{ left: number; top: number }}
 */
export const positionWikidotHovertip = (pointer, tip, viewport) => {
  let top = pointer.pageY
  if (
    viewport.height - pointer.clientY < tip.height + 2 * POINTER_GAP &&
    pointer.clientY > tip.height + 1.5 * POINTER_GAP
  ) {
    top -= tip.height + 1.5 * POINTER_GAP
  }
  top += POINTER_GAP
  let left = pointer.clientX - tip.width * 0.5
  if (left + tip.width > viewport.width - POINTER_GAP) {
    left = viewport.width - tip.width - POINTER_GAP
  }
  if (left < POINTER_GAP) left = POINTER_GAP
  return { left, top }
}

/**
 * Port of `OZONE.dialog.hovertip.dominit(root)` plus its mouse handlers.
 *
 * @param {Document} document
 * @returns {() => void}
 */
export const installWikidotHovertips = (document) => {
  const window = document.defaultView
  const body = document.body
  if (!window || !body) return () => {}

  let container = document.getElementById("odialog-hovertips")
  if (!container) {
    container = document.createElement("div")
    container.id = "odialog-hovertips"
    container.style.position = "absolute"
    container.style.zIndex = "100"
    container.style.top = "0px"
    container.style.width = "100%"
    body.appendChild(container)
  }

  /** @type {(() => void)[]} */
  const cleanups = []
  let pending = [...body.getElementsByTagName("div")].filter(
    (element) => element.id.endsWith("-hovertip") && element.parentElement !== container
  )

  /** @param {HTMLElement} tip */
  const attach = (tip) => {
    const control = document.getElementById(tip.id.replace(/-hovertip$/u, ""))
    if (!control || control === tip) return false

    if (!/hovertip/u.test(tip.className)) tip.className = `hovertip ${tip.className}`
    const hasContent = [...tip.getElementsByTagName("div")].some((element) =>
      element.classList.contains("content")
    )
    if (!hasContent) tip.innerHTML = `<div class="content">${tip.innerHTML}</div>`
    tip.style.position = "absolute"
    tip.style.display = "none"
    tip.style.border = "1px solid black"
    if (control.tagName.toLowerCase() !== "a") control.style.cursor = "help"
    container.appendChild(tip)

    /** @param {MouseEvent} event */
    const move = (event) => {
      const containerBox = container.getBoundingClientRect()
      const { left, top } = positionWikidotHovertip(
        event,
        { width: tip.offsetWidth, height: tip.offsetHeight },
        {
          width: document.documentElement.clientWidth,
          height: document.documentElement.clientHeight
        }
      )
      // YAHOO.util.Dom.setXY places the tip at page coordinates relative to
      // its positioned container.
      tip.style.left = `${left - (containerBox.left + window.scrollX)}px`
      tip.style.top = `${top - (containerBox.top + window.scrollY)}px`
    }
    /** @param {MouseEvent} event */
    const over = (event) => {
      tip.style.display = "block"
      move(event)
    }
    const out = () => {
      tip.style.display = "none"
    }
    control.addEventListener("mouseover", over)
    control.addEventListener("mousemove", move)
    control.addEventListener("mouseout", out)
    cleanups.push(() => {
      control.removeEventListener("mouseover", over)
      control.removeEventListener("mousemove", move)
      control.removeEventListener("mouseout", out)
    })
    return true
  }

  const attachPending = () => {
    pending = pending.filter((tip) => !attach(tip))
  }
  attachPending()

  // Attach tips for compatible controls that are added after initialization.
  const observer = new window.MutationObserver(() => {
    if (pending.length) attachPending()
  })
  observer.observe(body, { childList: true, subtree: true })

  return () => {
    observer.disconnect()
    for (const cleanup of cleanups) cleanup()
  }
}
