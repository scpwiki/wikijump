export class Suggestion {
  constructor(public text: string, public kind: string) {}

  replace(text = this.text, kind = this.kind) {
    return new Suggestion(text, kind)
  }
}

export class MultiWordSuggestion {
  constructor(public words: string[], public kind: string, public allowDash = true) {}

  stringify(seperator = " ") {
    return new Suggestion(this.words.join(seperator), this.kind)
  }
}
