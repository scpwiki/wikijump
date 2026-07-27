export class UserLocaleObserver {
  declare private target: HTMLElement
  declare private opts: App.PageData

  constructor(target: HTMLElement, opts: App.PageData) {
    this.target = target

    this.setLang(opts)
  }

  private setLang(opts: App.PageData) {
    this.target.lang = (
      opts.user_session?.user?.locales?.[0] ?? opts.site.locale
    )?.replaceAll("_", "-")
  }

  update(opts: App.PageData) {
    this.setLang(opts)
  }

  destroy() {
    this.target.lang = ""
  }
}
