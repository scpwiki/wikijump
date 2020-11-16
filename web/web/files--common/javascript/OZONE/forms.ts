declare const YAHOO: any;

export const forms = {
  lengthLimiter: class {
    /**
     * An object representing a length limiter for a text box.
     *
     * This function is supposed be called with `new`, but so far as I can
     * tell, the only module that actually uses the return value from that is
     * ManagerSiteLicenseModule.js. TODO Investigate further, possibly outsource
     * this function
     *
     * @param textElementId: The ID of the element whose content length should
     * be checked (input/textarea)
     * @param countElement: The ID of the element that contains text
     * representing the number of characters remaining.
     * @param limit: The character limit on the text box.
     */
    textElement: HTMLInputElement | HTMLTextAreaElement;
    countElement: HTMLElement;
    limit: number;

    constructor (
      textElementId: string,
      countElementId: string,
      limit: number
    ) {
      const textElement = document.getElementById(textElementId);
      // Assert that the text element exists and that it has a value property,
      // which indicates it is either an input or a textarea
      if (textElement === null || !('value' in textElement)) {
        throw new Error(`${textElementId} does not exist`);
      }
      this.textElement = textElement;

      const countElement = document.getElementById(countElementId);
      if (countElement === null) {
        throw new Error(`${countElementId} does not exist`);
      }
      this.countElement = countElement;

      this.limit = limit;
      YAHOO.util.Event.addListener(this.textElement, 'keyup', this.keyListener, this, true);
      this.keyListener();
    }

    keyListener (): void {
      /**
       * Function called after every key to truncate the form text.
       */
      let chars = this.textElement.value.replace(/\r\n/, '\n').length;
      this.countElement.innerHTML = `${this.limit - chars}`;
      if (chars > this.limit) {
        const scrollTop = this.textElement.scrollTop;
        this.textElement.value = this.textElement.value.substr(0, this.limit);
        this.textElement.scrollTop = scrollTop;
        chars = this.textElement.value.replace(/\r\n/, '\n').length;
        this.countElement.innerHTML = `${this.limit - chars}`;
      }
    }
  }
};
