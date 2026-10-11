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
      entry.history_kind !== "page" ||
      entry.page_revision_number === null ||
      entry.page_revision_number === undefined ||
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
