Basic lists were sourced from [SymSpell's dictionaries.](https://github.com/wolfgarbe/SymSpell/tree/master/SymSpell.FrequencyDictionary)

The English dictionary was made by merging a SymSpell dictionary and a generated word list from http://wordlist.aspell.net/.

The `wordlist.aspell.net` word list was generated with the following license:
```
Custom wordlist generated from http://app.aspell.net/create using SCOWL
with parameters:
  diacritic: both
  max_size: 95
  max_variant: 1
  special: hacker roman-numerals
  spelling: US GBs GBz CA AU

Using Git Commit From: Mon Dec 7 20:14:35 2020 -0500 [5ef55f9]

Copyright 2000-2019 by Kevin Atkinson

  Permission to use, copy, modify, distribute and sell these word
  lists, the associated scripts, the output created from the scripts,
  and its documentation for any purpose is hereby granted without fee,
  provided that the above copyright notice appears in all copies and
  that both that copyright notice and this permission notice appear in
  supporting documentation. Kevin Atkinson makes no representations
  about the suitability of this array for any purpose. It is provided
  "as is" without express or implied warranty.

Copyright (c) J Ross Beresford 1993-1999. All Rights Reserved.

  The following restriction is placed on the use of this publication:
  if The UK Advanced Cryptics Dictionary is used in a software package
  or redistributed in any form, the copyright notice must be
  prominently displayed and the text of this document must be included
  verbatim.

  There are no other restrictions: I would like to see the list
  distributed as widely as possible.

Special credit also goes to Alan Beale <biljir@pobox.com> as he has
given me an incredible amount of feedback and created a number of
special lists (those found in the Supplement) in order to help improve
the overall quality of SCOWL.

Many sources were used in the creation of SCOWL, most of them were in
the public domain or used indirectly.  For a full list please see the
SCOWL readme.

http://wordlist.aspell.net/
```
If you wish to do something similar with English or any other language, you can modify/examine this code if you wish:
```js
const fs = require('fs/promises');

const IGNORE_INITIALISMS = true
const FILE_TO_DEDUPLICATE = "./en-sloppy.txt"
const FILE_TO_WRITE = "./en-merged.txt"

async function merge() {
  const file = await fs.readFile(FILE_TO_DEDUPLICATE, "utf-8")
  const lines = file.replaceAll("\r\n", "\n").split("\n")

  const frequencies = new Map()
  const words = new Set()

  for (const line of lines) {
    let [word, frequency] = line.split(/\s+/)
    if (IGNORE_INITIALISMS && word.toUpperCase() === word) continue
    word = word.toLowerCase()
    if (!words.has(word)) words.add(word)
    if (!frequencies.has(word)) frequencies.set(word, frequency)
  }

  let output = ""
  for (const word of words) {
    if (!word) continue
    const frequency = frequencies.get(word)
    output += `${word} ${frequency}\n`
  }

  await fs.writeFile(FILE_TO_WRITE, output)
}

merge()
```
This will read a frequency dictionary and deduplicate all of its entries. This includes lowercasing them, so capitalized and uncapitalized variants will be deduplicated.
