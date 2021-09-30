import type CSS from "csstype";

import OZONE from ".";
import { ogettext } from "./loc";

declare const YAHOO: any;

/**
 * Ready-to-use dialog components.
 */

class Base {
  templateBase: string;
  template: string;
  title: null | string;
  buttons: string[];
  buttonObjects: { [buttonLabel: string]: HTMLAnchorElement };
  clickOutsideToClose: boolean;
  smooth: boolean;
  focusButton: null | string;
  buttonListeners: { [buttonLabel: string]: () => void };
  windowClass: string;
  content: string;
  dialogElement: null | HTMLDivElement;
  fixODate: boolean;
  style: CSS.Properties;

  constructor () {
    // ?? (unused)
    this.templateBase = '/common--dialogs/';
    this.template = '';
    // Title of the dialog, or null for no title
    this.title = null;
    // List of button labels (OZONE loc message IDs)
    this.buttons = [];
    // Should clicking outside this dialog close it?
    this.clickOutsideToClose = false;
    // Should this dialog animate smoothly?
    this.smooth = false;
    // Label of the button to focus when the dialog appears
    this.focusButton = null;
    // Map of button labels to callbacks
    this.buttonListeners = {};
    // CSS class(es) to apply to this dialog
    this.windowClass = '';
    // Text content of this dialog
    this.content = '';
    // Should span.odate inside the content be parsed to dates?
    this.fixODate = true;
    // CSS style to be applied
    this.style = {};

    // The element for the dialog itself
    this.dialogElement = null;
    // Container for button elements
    this.buttonObjects = {};
  }

  // TODO Make a function that adds both buttons and callbacks at
  // once, because this method is stupid

  addButtonListener (
    buttonLabel: string,
    eventListener: () => void,
    _oScope?: unknown
  ): void {
    /**
     * Attaches a callback to a button.
     *
     * Must be called for each button, otherwise the button won't do anything.
     *
     * @param buttonLabel: The label of the button to which to attach the
     * callback.
     * @param eventListener: The callback.
     * @param oScope: ?? (unused but some calls pass a value)
     */
    this.buttonListeners[buttonLabel] = eventListener;
  }

  show (): void {
    /**
     * Shows the dialog box.
     */
    // Construct a new element for the dialog and add class/style
    let dialogElement = document.createElement('div');
    this.dialogElement = dialogElement;
    dialogElement.className = `owindow ${this.windowClass}`;
    // Iterate over the style and attach them to the element
    Object.assign(dialogElement.style, this.style);

    // in there is a div class="content" - just place it inside, do not render
    // Construct a temporary element to analyse the content as HTML
    const tempElement = document.createElement('div');
    tempElement.innerHTML = this.content;
    if (
      tempElement.getElementsByTagName('div')[0] &&
      tempElement.getElementsByTagName('div')[0].classList.contains('owindow')
    ) {
      // If the dialog content is itself a dialog, bring it to the top
      // Assume content is already processed
      // XXX Class and style from before are wiped?
      dialogElement = tempElement.getElementsByTagName('div')[0];
    } else if (tempElement.querySelectorAll('div.content').length === 1) {
      // If the top layer is div.content, bring it to the top and skip the
      // content generation step
      // Assume content is already processed
      dialogElement.innerHTML = tempElement.innerHTML;
    } else {
      // Otherwise, the content needs processing
      if (this.title !== null) {
        const titleDiv = document.createElement('div');
        titleDiv.className = 'title';
        titleDiv.innerHTML = this.title;
        dialogElement.appendChild(titleDiv);
      }
      const contentDiv = document.createElement('div');
      contentDiv.className = 'content';
      contentDiv.innerHTML = this.content;
      if (this.fixODate) {
        OZONE.utils.formatDates(contentDiv);
      }
      dialogElement.appendChild(contentDiv);

      if (this.buttons.length > 0) {
        // If there are buttons, add them
        const buttonBar = document.createElement('div');
        buttonBar.className = 'button-bar';
        this.buttons.forEach(buttonLabel => {
          const button = document.createElement('a');
          button.href = "javascript:;";
          button.innerHTML = ogettext(buttonLabel);
          button.className = `button button-${buttonLabel.toLowerCase().replace(/ /g, '-')}`;
          if (buttonLabel in this.buttonListeners) {
            YAHOO.util.Event.addListener(button, 'click', this.buttonListeners[buttonLabel], this, true);
          }
          buttonBar.appendChild(button);
          this.buttonObjects[buttonLabel] = button;
        });
        dialogElement.appendChild(buttonBar);
      }
    }
    // Create a shader and show it
    OZONE.dialog.factory.shader().show();
    const container = OZONE.dialog.factory.boxcontainer();
    container.setContent(dialogElement);
    if (this.smooth) {
      container.showContent({ smooth: true });
    } else {
      container.showContent();
    }
    if (this.clickOutsideToClose) {
      container.clickOutsideToHide();
    }
    if (this.focusButton && this.buttonObjects[this.focusButton]) {
      // Focus the indicated button
      this.buttonObjects[this.focusButton].focus();
    }
  }

  hide (): void {
    /**
     * If smooth is set, fades out the dialog, otherwise does nothing.
     */
    if (this.smooth === true) {
      // @ts-expect-error Need to find a new animations library
      const ef = new fx.Opacity(this.dialogElement, { duration: 100 });
      ef.custom(1, 0);
    }
  }

  close (): void {
    /**
     * Close the dialog and clear the shader.
     */
    this.hide();
    OZONE.dialog.cleanAll({ timeout: 200 });
  }
}

class SmallInfoBox extends Base {
  /**
   * ?? (unused except in deprecated module ForumViewThreadModule.old.js)
   */
  constructor () {
    super();
    this.smooth = true;
    this.windowClass = "o-infobox";
  }
}

class WaitBox extends Base {
  /**
   * Inform the user about a lengthy process.
   *
   * No buttons to close the box by default - one must be added, all dialogs
   * must be cleared, or the page must be reloaded to clear it.
   */
  constructor () {
    super();
    this.smooth = true;
    this.windowClass = "owait";
  }
}

class SuccessBox extends Base {
  /**
   * Quickly inform the user about a successful operation.
   *
   * Unlike SuccessDialog, does not require the user's attention; the box will
   * disappear automatically.
   */
  timeout: number;

  constructor () {
    super();
    this.smooth = true;
    this.windowClass = "osuccess";
    this.timeout = 1500;
  }

  show (): void {
    super.show();
    if (this.timeout) {
      setTimeout(() => OZONE.dialog.cleanAll(), this.timeout);
    }
  }
}

class ErrorDialog extends Base {
  /**
   * Inform the user about an error.
   */
  constructor () {
    super();
    this.windowClass = "error";
    this.title = "Error";
    const lab = 'close message';
    this.buttons = [lab];
    this.addButtonListener(lab, this.close);
    this.focusButton = lab;
  }
}

class ConfirmationDialog extends Base {
  /**
   * Prompt the user to confirm an action.
   */
  constructor () {
    super();
    this.windowClass = "confirmation";
    this.title = "Confirmation";
  }
}

class SuccessDialog extends Base {
  /**
   * Notifications about a successful operation.
   */
  constructor () {
    super();
    this.smooth = true;
    this.windowClass = "confirm";
    this.title = "Success";
    this.buttons = ['close message'];
    this.addButtonListener('close message', this.close);
    this.focusButton = 'close message';
  }
}

class InfoDialog extends Base {
  /**
   * Displaying information.
   */
  constructor () {
    super();
    this.smooth = true;
    this.windowClass = "info";
    this.title = " ";
    this.buttons = ['close window'];
    this.addButtonListener('close window', this.close);
    this.focusButton = 'close window';
  }
}

class Dialog extends Base {
  /**
   * Just a basic dialog.
   *
   * Wikidot says "use this if unsure"
   */
  constructor () {
    super();
    this.title = '';
  }
}

export const dialogs = {
  Base,
  SmallInfoBox,
  WaitBox,
  SuccessBox,
  ErrorDialog,
  ConfirmationDialog,
  SuccessDialog,
  InfoDialog,
  Dialog
};
