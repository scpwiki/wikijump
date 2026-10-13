/**
 * A history entry is source-comparable only when it is a page-source
 * revision with a backing page revision number. File/attachment events are
 * timeline entries, not page source revisions, so the revision-diff
 * selector must never offer them as comparison endpoints. Refs #2143.
 *
 * @param {{
 *   history_kind: string
 *   timeline_number: number
 *   page_revision_number: number | null
 * }} entry
 * @returns {boolean} Whether the entry may be compared as page source.
 */
export const isComparableHistoryRevision = (entry) =>
  entry?.history_kind === "page" &&
  entry?.page_revision_number !== null &&
  entry?.page_revision_number !== undefined

/**
 * Timeline numbers the revision-diff selector may offer, oldest first.
 *
 * @param {{
 *   history_kind: string
 *   timeline_number: number
 *   page_revision_number: number | null
 * }[]} entries
 * @returns {number[]} Comparable page-revision timeline numbers.
 */
export const comparableHistoryTimelineNumbers = (entries) =>
  [...entries]
    .filter(isComparableHistoryRevision)
    .map((entry) => entry.timeline_number)
    .sort((a, b) => a - b)

/**
 * Choose the newest two distinct page-source revisions from a mixed
 * history timeline. File events remain visible in the timeline, but they
 * are not page source revisions and cannot be sent to the revision-diff
 * action directly.
 *
 * @param {{
 *   history_kind: string
 *   timeline_number: number
 *   page_revision_number: number | null
 * }[]} entries
 * @returns {[number, number] | undefined} Older and newer timeline
 *   numbers.
 */
export const defaultHistoryComparePair = (entries) => {
  const seenPageRevisions = new Set()
  const timelineNumbers = []

  for (const entry of [...entries].sort(
    (a, b) => b.timeline_number - a.timeline_number
  )) {
    if (
      !isComparableHistoryRevision(entry) ||
      seenPageRevisions.has(entry.page_revision_number)
    ) {
      continue
    }

    seenPageRevisions.add(entry.page_revision_number)
    timelineNumbers.push(entry.timeline_number)
    if (timelineNumbers.length === 2) break
  }

  if (timelineNumbers.length < 2) return undefined
  return [timelineNumbers[1], timelineNumbers[0]]
}
