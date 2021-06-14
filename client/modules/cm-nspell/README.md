# cm-nspell

An extension for CodeMirror 6 that adds spellchecking using the NSpell library.

### Languages

The chosen list of languages was based off of the [SCP branch list](http://o5command-int.wikidot.com/branch-list).

The extension currently supports (in no particular order):
* `en` English
* `de` German
* `es` Spanish
* `fr` French
* `it` Italian
* `ru` Russian
* `ko` Korean
* `pl` Polish
* `uk` Ukrainian
* `pt` Portuguese
* `cs` Czech
* `vi` Vietnamese
* `el` Greek
* `tr` Turkish
* `da` Danish
* `nb` Norwegian Bokmål
* `nn` Norwegian Nynorsk
* `sv` Swedish
* `fo` Faroese
* `nl` Dutch
* `hu` Hungarian
* `ro` Romanian

The following languages are missing, if going by the branch list:
* `zh`
* `ja`
* `th`
* `cy`
* `az`
* `et`
* `id`
* `bs`
* `hr`
* `sr`
* `sl`
* `sq`

`zh` and `ja` would require a custom solution to be spellchecked. The rest are not added due to a lack of dictionary or that it simply hasn't been done yet.

### Issues

* `de`: Compound words aren't recognized very well, causing many false positives.
* `it`: Italian can't be loaded due to the enormous amount of words its elision system generates, which consumes too much memory.
* `ko`: Seemingly doesn't work, despite loading.
* `hu`: Runs out of memory.
