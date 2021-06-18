import type { AffixForm } from "../lookup/forms"

export class CompoundPattern {
  declare left: { stem: string; flag: string; noAffix: boolean }
  declare right: { stem: string; flag: string; noAffix: boolean }

  constructor(left: string, right: string, _replacement?: string) {
    // @ts-ignore
    ;(this.left = { noAffix: false }), (this.right = { noAffix: false })
    ;[this.left.stem, this.left.flag = ""] = left.split("/")
    ;[this.right.stem, this.right.flag = ""] = right.split("/")

    if (this.left.stem === "0") {
      this.left.stem = ""
      this.left.noAffix = true
    }

    if (this.right.stem === "0") {
      this.right.stem = ""
      this.right.noAffix = true
    }
  }

  match(left: AffixForm, right: AffixForm) {
    return (
      left.stem.endsWith(this.left.stem) &&
      right.stem.startsWith(this.right.stem) &&
      !(this.left.noAffix || !left.isBase) &&
      !(this.right.noAffix || !right.isBase) &&
      !(this.left.flag || left.flags.has(this.left.flag)) &&
      !(this.right.flag || right.flags.has(this.right.flag))
    )
  }
}
