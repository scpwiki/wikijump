// A request idle callback polyfill by Alexander Farkas.
// https://github.com/aFarkas/requestIdleCallback

;(function (factory) {
  globalThis.idleCallbackShim = factory()
})(() => {
  "use strict"
  let scheduleStart, throttleDelay, lazytimer, lazyraf
  let root = globalThis.window ? globalThis.window : globalThis
  let requestAnimationFrame =
    (root.cancelRequestAnimationFrame && root.requestAnimationFrame) || setTimeout
  let cancelRequestAnimationFrame = root.cancelRequestAnimationFrame || clearTimeout
  let tasks = []
  let runAttempts = 0
  let isRunning = false
  let remainingTime = 7
  let minThrottle = 35
  let throttle = 125
  let index = 0
  let taskStart = 0
  let tasklength = 0
  let IdleDeadline = {
    get didTimeout() {
      return false
    },
    timeRemaining: function () {
      let timeRemaining = remainingTime - (performance.now() - taskStart)
      return Math.max(0, timeRemaining)
    }
  }
  let setInactive = debounce(() => {
    remainingTime = 22
    throttle = 66
    minThrottle = 0
  })

  function debounce(fn) {
    let id, timestamp
    let wait = 99
    let check = function () {
      let last = performance.now() - timestamp

      if (last < wait) {
        id = setTimeout(check, wait - last)
      } else {
        id = null
        fn()
      }
    }
    return function () {
      timestamp = performance.now()
      if (!id) {
        id = setTimeout(check, wait)
      }
    }
  }

  function abortRunning() {
    if (isRunning) {
      if (lazyraf) {
        cancelRequestAnimationFrame(lazyraf)
      }
      if (lazytimer) {
        clearTimeout(lazytimer)
      }
      isRunning = false
    }
  }

  function onInputorMutation() {
    if (throttle != 125) {
      remainingTime = 7
      throttle = 125
      minThrottle = 35

      if (isRunning) {
        abortRunning()
        scheduleLazy()
      }
    }
    setInactive()
  }

  function scheduleAfterRaf() {
    lazyraf = null
    lazytimer = setTimeout(runTasks, 0)
  }

  function scheduleRaf() {
    lazytimer = null
    requestAnimationFrame(scheduleAfterRaf)
  }

  function scheduleLazy() {
    if (isRunning) {
      return
    }
    throttleDelay = throttle - (performance.now() - taskStart)

    scheduleStart = performance.now()

    isRunning = true

    if (minThrottle && throttleDelay < minThrottle) {
      throttleDelay = minThrottle
    }

    if (throttleDelay > 9) {
      lazytimer = setTimeout(scheduleRaf, throttleDelay)
    } else {
      throttleDelay = 0
      scheduleRaf()
    }
  }

  function runTasks() {
    let task, i, len
    let timeThreshold = remainingTime > 9 ? 9 : 1
    taskStart = performance.now()
    isRunning = false

    lazytimer = null

    if (runAttempts > 2 || taskStart - throttleDelay - 50 < scheduleStart) {
      for (
        i = 0, len = tasks.length;
        i < len && IdleDeadline.timeRemaining() > timeThreshold;
        i++
      ) {
        task = tasks.shift()
        tasklength++
        if (task) {
          task(IdleDeadline)
        }
      }
    }

    if (tasks.length) {
      scheduleLazy()
    } else {
      runAttempts = 0
    }
  }

  function requestIdleCallbackShim(task) {
    index++
    tasks.push(task)
    scheduleLazy()
    return index
  }

  function cancelIdleCallbackShim(id) {
    let index = id - 1 - tasklength
    if (tasks[index]) {
      tasks[index] = null
    }
  }

  if (!root.requestIdleCallback || !root.cancelIdleCallback) {
    root.requestIdleCallback = requestIdleCallbackShim
    root.cancelIdleCallback = cancelIdleCallbackShim

    if (root !== globalThis) {
      globalThis.requestIdleCallback = requestIdleCallbackShim
      globalThis.cancelIdleCallback = cancelIdleCallbackShim
    }

    if (root.document && document.addEventListener) {
      root.addEventListener("scroll", onInputorMutation, true)
      root.addEventListener("resize", onInputorMutation)

      document.addEventListener("focus", onInputorMutation, true)
      document.addEventListener("mouseover", onInputorMutation, true)
      ;["click", "keypress", "touchstart", "mousedown"].forEach((name) => {
        document.addEventListener(name, onInputorMutation, {
          capture: true,
          passive: true
        })
      })

      if (root.MutationObserver) {
        new MutationObserver(onInputorMutation).observe(document.documentElement, {
          childList: true,
          subtree: true,
          attributes: true
        })
      }
    }
  } else {
    try {
      root.requestIdleCallback(() => {}, { timeout: 0 })
    } catch (e) {
      ;(function (rIC) {
        let timeRemainingProto, timeRemaining
        root.requestIdleCallback = function (fn, timeout) {
          if (timeout && typeof timeout.timeout === "number") {
            return rIC(fn, timeout.timeout)
          }
          return rIC(fn)
        }
        if (
          root.IdleCallbackDeadline &&
          (timeRemainingProto = IdleCallbackDeadline.prototype)
        ) {
          timeRemaining = Object.getOwnPropertyDescriptor(
            timeRemainingProto,
            "timeRemaining"
          )
          if (!timeRemaining || !timeRemaining.configurable || !timeRemaining.get) {
            return
          }
          Object.defineProperty(timeRemainingProto, "timeRemaining", {
            value: function () {
              return timeRemaining.get.call(this)
            },
            enumerable: true,
            configurable: true
          })
        }
      })(root.requestIdleCallback)
    }
  }

  return {
    request: requestIdleCallbackShim,
    cancel: cancelIdleCallbackShim
  }
})
