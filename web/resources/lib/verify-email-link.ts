/**
 * Detects human interactivity and resolves a promise upon detection.
 * Starts listening upon instantiation.
 */
class HumanDetector {
  /** List of events that the dectector listens to. */
  static EVENTS = [
    "touchmove",
    "touchstart",
    "pointerdown",
    "pointermove",
    "keydown",
    "keyup"
  ]

  /** Tracks if an event has been detected yet. */
  private _detected = false

  /** Resolves the detector's promise. */
  private declare _resolve: () => void

  /** Promise that resolves when human interactivity is detected. */
  declare promise: Promise<void>

  /** Constructs a new detector. Starts listening to events automatically. */
  constructor() {
    this.detect = this.detect.bind(this)
    this.promise = new Promise<void>(resolve => {
      this._resolve = resolve
    })
    this.startListening()
  }

  /** True if human interactivity has been detected. */
  get detected() {
    return this._detected
  }

  /** Fired when human interactivity is detected. */
  private detect() {
    this._detected = true
    this._resolve()
    this.stopListening()
  }

  /** Starts listening for human interactivity. */
  private startListening() {
    for (const event of HumanDetector.EVENTS) {
      // eslint-disable-next-line @typescript-eslint/unbound-method
      document.addEventListener(event, this.detect)
    }
  }

  /** Stops listening for human interactivity. */
  private stopListening() {
    for (const event of HumanDetector.EVENTS) {
      // eslint-disable-next-line @typescript-eslint/unbound-method
      document.removeEventListener(event, this.detect)
    }
  }
}

async function verifyEmailLink() {
  const divWaiting = document.getElementById("verify_waiting")
  const divPleaseInteract = document.getElementById("verify_please_interact")
  const divSuccess = document.getElementById("verify_success")
  const divFailure = document.getElementById("verify_failure")

  if (!divWaiting || !divPleaseInteract || !divSuccess || !divFailure) {
    throw new Error("Missing required DOM elements.")
  }

  const detector = new HumanDetector()

  const timeout = setTimeout(() => {
    divPleaseInteract.style.display = "block"
  }, 5000)

  await detector.promise

  clearTimeout(timeout)

  // POST at our own URL to verify the email link
  const res = await fetch(window.location.href, { method: "POST" })

  if (res.ok) {
    divWaiting.style.display = "none"
    divPleaseInteract.style.display = "none"
    divSuccess.style.display = "block"
    divFailure.style.display = "none"
  } else {
    divWaiting.style.display = "none"
    divPleaseInteract.style.display = "none"
    divSuccess.style.display = "none"
    divFailure.style.display = "block"
  }
}

document.addEventListener("DOMContentLoaded", verifyEmailLink)

export {}
