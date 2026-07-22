export const WIKIDOT_STANDARD_LICENSE_OPTIONS = [
  ["cc-by-sa-3.0", "Creative Commons Attribution-ShareAlike 3.0 License (recommended)"],
  ["cc-by-3.0", "Creative Commons Attribution 3.0 License"],
  ["cc-by-nd-3.0", "Creative Commons Attribution-NoDerivs 3.0 License"],
  ["cc-by-nc-3.0", "Creative Commons Attribution-NonCommercial 3.0 License"],
  [
    "cc-by-nc-sa-3.0",
    "Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License"
  ],
  ["cc-by-nc-nd-3.0", "Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License"],
  ["cc-by-sa-2.5", "Creative Commons Attribution Share Alike 2.5"],
  ["cc-by-2.5", "Creative Commons Attribution 2.5"],
  ["cc-by-nd-2.5", "Creative Commons Attribution No Derivatives 2.5"],
  ["cc-by-nc-2.5", "Creative Commons Attribution Non-commercial 2.5"],
  ["cc-by-nc-sa-2.5", "Creative Commons Attribution Non-commercial Share Alike 2.5"],
  ["cc-by-nc-nd-2.5", "Creative Commons Attribution Non-commercial No Derivatives 2.5"],
  ["gnu-fdl-1.2", "GNU Free Documentation License 1.2"]
].map(([value, label]) => ({ value, label }))

export const WIKIDOT_SPECIAL_LICENSE_OPTIONS = [
  { value: "other", label: "Other" },
  { value: "copyright", label: "Standard copyright (not recommended)" }
]

export const WIKIDOT_LICENSE_OPTIONS = [
  ...WIKIDOT_STANDARD_LICENSE_OPTIONS,
  ...WIKIDOT_SPECIAL_LICENSE_OPTIONS
]

/** @param {string} currentLicense */
export const licenseOptionsFor = (currentLicense) =>
  WIKIDOT_LICENSE_OPTIONS.some(({ value }) => value === currentLicense)
    ? WIKIDOT_LICENSE_OPTIONS
    : [...WIKIDOT_LICENSE_OPTIONS, { value: currentLicense, label: currentLicense }]

/**
 * @param {{
 *   category_id: number
 *   slug: string
 *   license: string | null
 *   license_other?: string | null
 * }} category
 * @param {string} defaultLicense
 * @param {string | null | undefined} defaultLicenseOther
 */
export const licenseFormValues = (
  category,
  defaultLicense,
  defaultLicenseOther = null
) => ({
  categoryId: category.category_id,
  inherit: category.slug !== "_default" && category.license === null,
  license: category.license ?? defaultLicense,
  licenseOther:
    category.license_other ??
    (category.license === null ? (defaultLicenseOther ?? "") : "")
})

/** @param {{ inherit: boolean; license: string; licenseOther?: string }} form */
export const licenseUpdateValue = (form) => ({
  license: form.inherit ? null : form.license,
  licenseOther:
    !form.inherit && form.license === "other" ? (form.licenseOther ?? "") : null
})
