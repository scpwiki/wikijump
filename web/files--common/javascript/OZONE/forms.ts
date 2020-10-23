export const forms = {
  lengthLimiter: function (
    textElementId: string,
    countElementId: string,
    limit: number
  ): void {
    /**
     * Constructs an object representing a length limiter for a text box.
     *
     * This function is supposed be called with `new`, but so far as I can
     * tell, the only module that actually uses the return value from that is
     * ManagerSiteLicenseModule.js. TODO Investigate further, possibly
     * refactor this into a class, possibly outsource this function
     *
     * @param textElementId: The ID of the element who content length should
     * be checked (input/textarea)
     * @param countElement: The ID of the element that contains text
     * representing the number of characters remaining.
     * @param limit: The character limit on the text box.
     */
    this.keyListener = () => {
      /**
       * Function called after every key to truncate the form text.
       */
      // get number of characters...
      let chars = this.textElement.value.replace(/\r\n/, '\n').length;
      this.countElement.innerHTML = this.limit - chars;
      if (chars > this.limit) {
        const scrollTop = this.textElement.scrollTop;
        this.textElement.value = this.textElement.value.substr(0, this.limit);
        this.textElement.scrollTop = scrollTop;
        chars = this.textElement.value.replace(/\r\n/, '\n').length;
        this.countElement.innerHTML = this.limit - chars;
      }
    };
    this.textElement = document.getElementById(textElementId);
    this.countElement = document.getElementById(countElementId);
    this.limit = limit;

    YAHOO.util.Event.addListener(this.textElement, 'keyup', this.keyListener, this, true);
    this.keyListener();
  }
};
