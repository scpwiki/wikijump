/**
 * Convenience function that converts date string format provided by
 * Deepwell as `YYYY-MM-DD HH:MM:SS.SSSSSS +HH:MM:SS` into
 * `YYYY-MM-DDTHH:mm:ss.sss+HH:mm`
 */
export function convertDateString(dateStr: string): string {
  let dateParts = dateStr.split(" ")
  let time = dateParts[1].split(".")
  let subsec = time.length == 2 ? "." + time[1].slice(0, 3) : ""
  let timezone = dateParts[2].split(":")
  timezone.pop()
  let output = `${dateParts[0]}T${time[0]}${subsec}${timezone.join(":")}`
  return output
}

/**
 * Convenience function that converts date string format provided by
 * Deepwell as `YYYY-MM-DD HH:MM:SS.SSSSSS +HH:MM:SS` into unix epoch
 */
export function parseDateEpoch(dateStr: string): number {
  return Date.parse(convertDateString(dateStr))
}

/**
 * Convenience function that converts date string format provided by
 * Deepwell as `YYYY-MM-DD HH:MM:SS.SSSSSS +HH:MM:SS` into date object
 */
export function parseDate(dateStr: string): Date {
  return new Date(convertDateString(dateStr))
}
