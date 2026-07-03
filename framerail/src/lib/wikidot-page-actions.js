/**
 * @typedef {object} WikidotPageActionLabels
 * @property {string} rate
 * @property {string | null} ratingText
 * @property {string} discuss
 */

/**
 * @param {readonly string[] | null | undefined} tags
 * @returns {boolean}
 */
export const isWikidotFragmentPage = (tags) => tags?.includes("fragment") ?? false

/**
 * @param {{ rating?: number | null; comments?: number | null }} snapshot
 * @returns {WikidotPageActionLabels}
 */
export const buildWikidotPageActionLabels = ({ rating, comments }) => {
  const ratingText = rating === null || rating === undefined ? null : formatSigned(rating)

  return {
    ratingText,
    rate: ratingText === null ? "Rate" : `Rate (${ratingText})`,
    discuss:
      comments === null || comments === undefined ? "Discuss" : `Discuss (${comments})`
  }
}

/**
 * @param {number} value
 * @returns {string}
 */
export const formatSigned = (value) => {
  return value > 0 ? `+${value}` : `${value}`
}
