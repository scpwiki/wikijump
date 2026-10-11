const reverse = (value) => [...value].reverse().join("")

const EMAIL_ADDRESS = /^[A-Z0-9._%+'-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/iu

/**
 * Decode the client-side format used for plain Wikidot email links.
 *
 * @param {string} value
 * @returns {{ address: string; label: string } | null}
 */
export const decodeWikidotEmailObfuscation = (value) => {
  const separator = value.indexOf("#")
  if (separator <= 0 || separator === value.length - 1) return null

  const encodedAddress = value.slice(0, separator)
  const [reversedDomain, reversedLocal, ...extra] = encodedAddress.split("|")
  if (!reversedDomain || !reversedLocal || extra.length > 0) return null

  const address = `${reverse(reversedLocal)}@${reverse(reversedDomain)}`
  if (!EMAIL_ADDRESS.test(address)) return null

  const encodedLabel = value.slice(separator + 1)
  const label = encodedLabel === encodedAddress ? address : reverse(encodedLabel)
  if (!label.trim()) return null

  return { address, label }
}

/**
 * Replace recognized scrambled email spans with safe, visible mailto
 * anchors.
 *
 * @param {Element} root
 */
const restoreIn = (root) => {
  const spans = [
    ...(root.matches?.(".wiki-email") ? [root] : []),
    ...root.querySelectorAll(".wiki-email")
  ]

  for (const span of spans) {
    if (span.childNodes.length !== 1 || span.firstChild?.nodeType !== Node.TEXT_NODE) {
      continue
    }

    const decoded = decodeWikidotEmailObfuscation(span.textContent ?? "")
    if (!decoded) continue

    const link = document.createElement("a")
    link.href = `mailto:${decoded.address}`
    link.textContent = decoded.label
    span.replaceChildren(link)
    span.style.visibility = "visible"
  }
}

/**
 * Restore email spans in the initial article and on subsequent client
 * updates.
 *
 * @param {Element} root
 * @returns {() => void}
 */
export const observeWikidotEmailObfuscation = (root) => {
  restoreIn(root)

  const observer = new MutationObserver((records) => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (node instanceof Element) restoreIn(node)
      }
    }
  })
  observer.observe(root, { childList: true, subtree: true })

  return () => observer.disconnect()
}

/** @type {import("svelte/action").Action<Element>} */
export const wikidotEmailObfuscation = (root) => observeWikidotEmailObfuscation(root)
