// based on https://github.com/antimatter15/heapqueue.js/blob/master/heapqueue.js

/**
 * Wrapper around a normal array that uses a comparator function to ensure
 * that the lowest index in the array is of the minimum priority.
 *
 * @typeParam T - The type of object to be stored in the list.
 */
export class PriorityList<T> {
  /** The comparator function that is used to sort the list. */
  declare cmp: (a: T, b: T) => number

  /** The actual list that is being wrapped around. */
  declare data: T[]

  /**
   * @param cmp - The comparator function that the list will be sorted
   *   with. Returning a negative value from this function indicates that
   *   `a` has a lower priority than `b`, while a positive value indicates
   *   that `a` has a higher priority than `b`.
   */
  constructor(cmp: (a: T, b: T) => number) {
    this.cmp = cmp
    this.data = []
  }

  /** Length of the list. */
  get length() {
    return this.data.length
  }

  /** Returns the lowest priority in the list. */
  peek() {
    return this.data[0]
  }

  /**
   * Adds a new value to the list.
   *
   * @param val - The value to be added.
   */
  push(val: T) {
    this.data.push(val)

    let pos = this.data.length - 1

    while (pos > 0) {
      const parent = (pos - 1) >>> 1
      if (this.cmp(this.data[pos], this.data[parent]) < 0) {
        const x = this.data[parent]
        this.data[parent] = this.data[pos]
        this.data[pos] = x
        pos = parent
      } else {
        break
      }
    }

    return this.data.length
  }

  /** Removes the lowest priority item in the list. */
  pop() {
    const lastValue = this.data.pop()!
    let ret = this.data[0]

    if (this.data.length > 0) {
      this.data[0] = lastValue
      let pos = 0
      let last = this.data.length - 1

      while (true) {
        const left = (pos << 1) + 1
        const right = left + 1

        let minIndex = pos
        if (left <= last && this.cmp(this.data[left], this.data[minIndex]) < 0) {
          minIndex = left
        }
        if (right <= last && this.cmp(this.data[right], this.data[minIndex]) < 0) {
          minIndex = right
        }

        if (minIndex !== pos) {
          const x = this.data[minIndex]
          this.data[minIndex] = this.data[pos]
          this.data[pos] = x
          pos = minIndex
        } else {
          break
        }
      }
    } else {
      ret = lastValue
    }

    return ret
  }

  /**
   * Completely sorts the list using a comparator function and returns the
   * actual internal array.
   *
   * @param cmp - The comparator function to use. Defaults to the one the
   *   list was instantiated with.
   */
  sort(cmp = this.cmp) {
    return this.data.sort(cmp)
  }
}
