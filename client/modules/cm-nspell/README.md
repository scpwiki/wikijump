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

* `it`: Italian can't be loaded due to the enormous amount of words its elision system generates, which consumes too much memory.
* `ko`: Seemingly doesn't work, despite loading.
* `hu`: Runs out of memory.

### Vendored `de` Files

The `/vendor` folder contains four files:
* `de.aff`: A special affix file made from two sources: `transam45.gmx.net` (as in, that's the author, apparently), and [this GitHub repository](https://github.com/vpikulik/hunspell_de_compounds).
* `de-bjoern.dic`: A dictionary sourced from Björn Jacke's [Firefox extension](https://addons.mozilla.org/en-US/firefox/addon/german-dictionary-de_de-for-sp/?utm_source=addons.mozilla.org&utm_medium=referral&utm_content=search).
* `de-transam`: The specially processed dictionary file from the [previously mentioned GitHub repository](https://github.com/vpikulik/hunspell_de_compounds).
* `de-chrome`: A list of words added by the [Chromium developers](https://chromium.googlesource.com/chromium/deps/hunspell_dictionaries/+/refs/heads/main), albeit cleaned up and with a few words added or removed.

This combination of files presents a _much_ nicer experience with the German spellchecker, but is ultimately a bit of an experiment. If you actually speak German, please let us know how the spellchecker behaves for you.
