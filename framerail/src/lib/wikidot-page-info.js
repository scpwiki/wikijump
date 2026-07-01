const WIKIDOT_SOURCE_TIME_ZONE = "Asia/Tokyo"
const WIKIDOT_MONTHS = Object.freeze([
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec"
])

const wikidotDatePartsFormatter = new Intl.DateTimeFormat("en-US", {
  timeZone: WIKIDOT_SOURCE_TIME_ZONE,
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
  hour12: false
})

/**
 * @param {Intl.DateTimeFormatPart[]} parts
 * @param {string} type
 */
const datePart = (parts, type) => {
  return parts.find((part) => part.type === type)?.value ?? ""
}

/** @param {number} timestampMs */
export const formatWikidotSourceDate = (timestampMs) => {
  const parts = wikidotDatePartsFormatter.formatToParts(new Date(timestampMs))
  const day = Number(datePart(parts, "day"))
  const month = Number(datePart(parts, "month"))
  const year = datePart(parts, "year")
  const hour = datePart(parts, "hour").replace(/^24$/, "00").padStart(2, "0")
  const minute = datePart(parts, "minute").padStart(2, "0")

  return `${day} ${WIKIDOT_MONTHS[month - 1]} ${year}, ${hour}:${minute}`
}

/** @param {number} elapsedMs */
export const formatWikidotRelativeAge = (elapsedMs) => {
  const totalSeconds = Math.max(0, Math.floor(elapsedMs / 1000))

  if (totalSeconds < 60) return "less than a minute ago"

  const minutes = Math.floor(totalSeconds / 60)
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? "" : "s"} ago`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hour${hours === 1 ? "" : "s"} ago`

  const days = Math.floor(hours / 24)
  return `${days} day${days === 1 ? "" : "s"} ago`
}

/** @param {{ revision: number; updatedAt: string; now?: number }} input */
export const buildWikidotPageInfoText = ({ revision, updatedAt, now = Date.now() }) => {
  const updatedAtMs = Date.parse(updatedAt)

  if (!Number.isFinite(updatedAtMs)) return null

  const date = formatWikidotSourceDate(updatedAtMs)
  const relative = formatWikidotRelativeAge(now - updatedAtMs)

  return `page revision: ${revision}, last edited: ${date} (${relative})`
}
