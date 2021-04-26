// credit to: https://stackoverflow.com/a/23329386 (CC BY-SA 4.0)
/** Returns the length (in bytes, UTF-8) of a string. */
function byteLength(str: string) {
  let length = str.length
  for (let i = str.length - 1; i >= 0; i--) {
    const code = str.charCodeAt(i)
    if (code > 0x7f && code <= 0x7ff) length++
    else if (code > 0x7ff && code <= 0xffff) length += 2
    if (code >= 0xdc00 && code <= 0xdfff) i-- //trail surrogate
  }
  return length
}

export function createUTF8PositionMap(str: string) {
  // using an array is quite a bit faster than a Map here
  // additionally, we're doing a worst-case length * 3 preallocation here
  // this can significantly speedup populating the array because
  // the array never needs expanded memory
  const map = new Array<number>(str.length * 3)

  // position indexes
  let utf16 = 0
  let utf8 = 0
  for (const char of str) {
    const utf16len = char.length // won't be 1 for some characters (e.g. emojis)
    const utf8len = byteLength(char)
    for (let idx = 0; idx < utf8len; idx++) {
      map[utf8 + idx] = utf16
    }
    utf16 += utf16len
    utf8 += utf8len
  }
  return map
}
