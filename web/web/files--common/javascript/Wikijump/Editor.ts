import { compress, compressTight } from 'compress-tag';

import Wikijump from ".";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { ogettext } from "@/javascript/OZONE/loc";
import type { Wikirequest } from "wikirequest";

declare const YAHOO: any;
declare type YahooResponse = any;
declare const WIKIREQUEST: Wikirequest;

let INSERT_NEWLINE = "\n";

/**
 * Abstraction proxy for handling textarea range, cursor position etc.
 */
class TextElementProxyUtil {
  field: HTMLTextAreaElement;
  browserType: 'gecko' | 'ie' | null;

  constructor (fieldId: string) {
    this.field = document.getElementById(fieldId) as HTMLTextAreaElement;
    /* determine type of the browser. IE vs Gecko/Opera vs the rest.
     */
    this.browserType = null;
    this.detectBrowser();
  }

  detectBrowser () {
    if (this.field.selectionStart || this.field.selectionStart === 0) {
      this.browserType = "gecko";
    } else {
      this.field.focus();
      // @ts-expect-error Property is intended not to exist on modern browsers
      // XXX Good god this whole thing just has to go
      if (document.selection.createRange) {
        this.browserType = "ie";
        // also change newline character
        INSERT_NEWLINE = "\r\n";
      }
    }
  }

  getCursorPosition () {
    const range = this.getSelectionRange();
    return range[1]; // end of the selection should be considered cursor position
  }

  getSelectionRange () {
    /**
     * Gets the start and end indexes of the highlighted range in the text.
     */
    let startPos;
    let endPos;
    this.field.focus();
    if (this.browserType === "gecko") {
      startPos = this.field.selectionStart;
      endPos = this.field.selectionEnd;
    }
    if (this.browserType === "ie") {
      // @ts-expect-error IE-specific
      if (document.selection) {
        // @ts-expect-error IE-specific
        const range = document.selection.createRange();
        const storedRange = range.duplicate();
        storedRange.moveToElementText(this.field);
        storedRange.setEndPoint('StartToStart', range);
        startPos = this.field.value.length - storedRange.text.length;
        endPos = startPos + range.text.length;
      }
    }
    this.field.focus();
    return [startPos, endPos];
  }

  setSelectionRange (startPos: number, endPos: number) {
    /**
     * Sets the selection range (aka highlighted text).
     *
     * @param startPos: The start index of the selection range.
     * @param endPos: The end index of the selection range.
     */
    this.field.focus();
    if (this.browserType === "gecko") {
      this.field.setSelectionRange(startPos, endPos);
    }
    if (this.browserType === "ie") {
      // fix position: Windows based "new lines" (\r\n) are counted as 2 characters,
      // but not when it comes to positioning the cursor!!!
      const beforeText = this.field.value.substring(0, startPos);
      const selText = this.field.value.substring(startPos, endPos);
      startPos = beforeText.replace(/\r\n/g, "\n").length;
      endPos = startPos + selText.replace(/\r\n/g, "\n").length;
      // @ts-expect-error IE-specific
      const range = this.field.createTextRange();
      range.collapse(true);
      range.moveEnd('character', endPos);
      range.moveStart('character', startPos);
      range.select();
    }
    this.field.focus();
  }

  trimSelection () {
    /**
     * Trims the selection to remove whitespaces from the begining or end of the selection.
     */
    const range = this.getSelectionRange();
    const selectionText = this.field.value.substring(range[0], range[1]);
    const trimLeft = selectionText.length - selectionText.replace(/^\s+/, "").length;
    const trimRight = selectionText.length - selectionText.replace(/\s+$/, "").length;
    this.setSelectionRange(range[0] + trimLeft, range[1] - trimRight);
  }
}

// This very much feels like it wants to be converted to a class, but not quite
// - init is called without new, and the editor is never saved to a variable
// but rather just attached to the page.
// Converting this to a class may make it possible to have more than one editor
// open at a time.

export const Editor = {
  editElementId: "",
  toolbarPanelId: "",
  ranger: null as null | TextElementProxyUtil,
  lastKeyCode: null,

  currentPos: 0, // required by IE not to lose position when opening a wizard window

  init: function (editElementId: string, toolbarPanelId: string): void {
    /**
     * Initialises a new editor.
     *
     * @param editElementId: The ID of the textarea.
     * @param toolbarPanelId: The ID of the element in which to place the
     * editor's toolbar.
     */
    Editor.editElementId = editElementId;
    Editor.toolbarPanelId = toolbarPanelId;
    Editor.ranger = new TextElementProxyUtil(editElementId);
    YAHOO.util.Event.addListener(this.editElementId, "keypress", Editor.keyboardListener);
    YAHOO.util.Event.addListener(this.editElementId, "keydown", function () { Editor.lastKeyCode = null; });
    YAHOO.util.Event.addListener(this.editElementId, "keyup", Editor.codeAssist.listener);

    let durl;
    switch (OZONE.lang) {
      case 'pl':
        durl = "/common--editor/dialogs-pl.html";
        break;
      default:
        durl = "/common--editor/dialogs.html";
    }

    // also read and add dialogs data
    YAHOO.util.Connect.asyncRequest('GET', durl, Editor.initCallback, null);

    // init newline character

    OZONE.loc.addMessage("cancel", "anuluj", "pl");
    OZONE.loc.addMessage("insert code", "wstaw kod", "pl");
    OZONE.loc.addMessage("Image wizard", "Magik wstawiania obrazu", "pl");
    OZONE.loc.addMessage("Table wizard", "Magik tabeli", "pl");
  },

  shutDown: function (): void {
    /**
     * Destroys the editor.
     */
    YAHOO.util.Event.removeListener(this.editElementId, "keypress", Editor.keyboardListener);
    YAHOO.util.Event.removeListener(this.editElementId, "keyup", Editor.codeAssist.listener);
    Editor.ranger = null;
    Editor.toolbarPanelId = "";
    Editor.editElementId = "";
  },

  initCallback: {
    success: function (response: { responseText: string }): void {
      const content = response.responseText;
      const div = document.createElement('div');
      div.id = "wd-ed-dialogs";
      div.innerHTML = content;
      div.style.display = "none";

      const body = document.getElementsByTagName('body')[0];
      body.appendChild(div);
      const etoolbar = document.getElementById("wd-ed-toolbar")!;
      const panel = document.getElementById(Editor.toolbarPanelId);
      if (panel) {
        panel.innerHTML = OZONE.utils.olang(etoolbar.innerHTML);
        const as = panel.getElementsByTagName('a');
        OZONE.dialog.hovertip.makeTip(as, { style: { width: 'auto' }, delay: 200 });
        Wikijump.page.fixers.fixMenu(Editor.toolbarPanelId);
      }
    },
    failure: function (_response: unknown): void {
      alert("failure error code\n823468008623487666624");
    }
  },

  /* buttons listeners */
  buttons: {
    bold: function (_event?: Event): void {
      Editor.utils.insertTags(
        "**", "**", "bold text", Editor.utils.trimSelection
      );
    },
    italic: function (_event?: Event): void {
      Editor.utils.insertTags(
        "//", "//", "italic text", Editor.utils.trimSelection
      );
    },
    underline: function (_event?: Event): void {
      Editor.utils.insertTags(
        "__", "__", "underline text", Editor.utils.trimSelection
      );
    },
    strikethrough: function (_event?: Event): void {
      Editor.utils.insertTags(
        "--", "--", "strikethrough text", Editor.utils.trimSelection
      );
    },
    teletype: function (_event?: Event): void {
      Editor.utils.insertTags(
        "{{", "}}", "teletype text", Editor.utils.trimSelection
      );
    },
    superscript: function (_event?: Event): void {
      Editor.utils.insertTags(
        "^^", "^^", "superscript", Editor.utils.trimSelection
      );
    },
    subscript: function (_event?: Event): void {
      Editor.utils.insertTags(
        ",,", ",,", "subscript", Editor.utils.trimSelection
      );
    },
    raw: function (_event?: Event): void {
      Editor.utils.insertTags(
        "@@", "@@", "raw text", Editor.utils.trimSelection
      );
    },

    heading: function (_event: Event, level: number): void {
      const pluses = "+".repeat(level);
      Editor.utils.insertTags(
        `${pluses} `, "", `heading level ${level}`,
        Editor.utils.trimSelection,
        Editor.utils.endWith2NewLine,
        Editor.utils.startWith2NewLine
      );
    },

    quote: function (_event?: Event): void {
      Editor.utils.insertTags(
        "> ", "", "quoted text",
        Editor.utils.processQuoteText,
        Editor.utils.endWithAtLeast1NewLine,
        Editor.utils.startWithAtLeast1NewLine);
    },
    hr: function (_event?: Event): void {
      Editor.utils.insertText(
        "------",
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine
      );
    },
    clearFloat: function (_event?: Event, dir?: '<' | '>'): void {
      let text = "~~~~";
      if (dir) text += dir;
      Editor.utils.insertText(
        text,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine
      );
    },
    toc: function (_event?: Event): void {
      Editor.utils.insertText(
        "[[toc]]",
        Editor.utils.endWithAtLeast1NewLine,
        Editor.utils.startWithAtLeast1NewLine
      );
    },
    uri: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[http://www.example.com ", "]", "describe link",
        Editor.utils.trimSelection
      );
    },
    pageLink: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[[", "]]]", "page name",
        Editor.utils.trimSelection
      );
    },
    math: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[math]]" + INSERT_NEWLINE, INSERT_NEWLINE + "[[/math]]", "insert LaTeX equation here",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine);
    },
    numberedList: function (_event?: Event): void {
      Editor.utils.insertTags(
        "# ", "", "list item",
        Editor.utils.processNumberedList,
        Editor.utils.endWithAtLeast1NewLine,
        Editor.utils.startWithAtLeast1NewLine);
    },
    bulletedList: function (_event?: Event): void {
      Editor.utils.insertTags(
        "* ", "", "list item",
        Editor.utils.processBulletedList,
        Editor.utils.endWithAtLeast1NewLine,
        Editor.utils.startWithAtLeast1NewLine);
    },
    definitionList: function (_event?: Event): void {
      Editor.utils.insertTags(
        ": ", " : definition", "item",
        Editor.utils.processBulletedList,
        Editor.utils.endWithAtLeast1NewLine,
        Editor.utils.startWithAtLeast1NewLine);
    },

    increaseListIndent: function (_event?: Event): void {
      Editor.utils.insertText('', Editor.utils.increaseListIndent);
    },
    decreaseListIndent: function (_event?: Event): void {
      Editor.utils.insertText('', Editor.utils.decreaseListIndent);
    },
    footnote: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[footnote]] ", " [[/footnote]]", "footnote text",
        Editor.utils.trimSelection
      );
    },
    inlineMath: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[$ ", " $]]", "insert LaTeX equation here",
        Editor.utils.trimSelection
      );
    },
    code: function (_event?: Event): void {
      Editor.utils.insertTags(
        `[[code]]${INSERT_NEWLINE}`,
        `${INSERT_NEWLINE}[[/code]]`,
        "insert the code here",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine);
    },
    video: function (_event?: Event): void {
      Editor.utils.insertTags(
        `[[embedvideo]]${INSERT_NEWLINE}`,
        `${INSERT_NEWLINE}[[/embedvideo]]`,
        "paste the html for the video here (Google Video, YouTube, Revver, Dailymotion)",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine);
    },
    audio: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[embedaudio]]" + INSERT_NEWLINE, INSERT_NEWLINE + "[[/embedaudio]]", "paste the html for the audio here (odeo)",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine);
    },
    image: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[image ", "]]", "source",
        Editor.utils.trimSelection
      );
    },
    div: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[div]]" + INSERT_NEWLINE, INSERT_NEWLINE + "[[/div]]", "block contents",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine);
    },
    bibliography: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[[bibliography]]" + INSERT_NEWLINE + ": ", " : full source reference" + INSERT_NEWLINE + "[[/bibliography]]", "label",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine);
    },
    bibliographycitation: function (_event?: Event): void {
      Editor.utils.insertTags(
        "[((bibcite ", "))]", "label",
        Editor.utils.trimSelection
      );
    },

    imageWizard: function (_event?: Event): void {
      Editor.currentPos = Editor.ranger!.getSelectionRange()[0];
      // open a dialog...
      const d = new OZONE.dialogs.Dialog();
      d.style.width = "70%";
      d.title = ogettext("Image wizard");
      d.buttons = ["cancel", "insert code"];
      d.addButtonListener("cancel", d.close);
      d.addButtonListener("insert code", Editor.imageWizard.insertCode);
      d.content = document.getElementById("wd-ed-imagewizard-dialog")!.innerHTML.replace(/-template/g, "");
      d.show();
      Editor.imageWizard.updateSourceBlock();
    },
    tableWizard: function (_event?: Event): void {
      Editor.currentPos = Editor.ranger!.getSelectionRange()[0];
      // open a dialog...
      const d = new OZONE.dialogs.Dialog();
      d.title = ogettext("Table wizard");
      d.buttons = ["cancel", "insert code"];
      d.addButtonListener("cancel", d.close);
      d.addButtonListener("insert code", Editor.listeners.tableWizardInsert);
      d.content = document.getElementById("wd-ed-tablewizard-dialog")!.innerHTML.replace(/-template/g, '');
      d.show();
    },
    uriWizard: function (_event?: Event): void {
      Editor.currentPos = Editor.ranger!.getSelectionRange()[0];
      // open a dialog...
      const d = new OZONE.dialogs.Dialog();
      d.title = ogettext("URL link wizard");
      d.buttons = ["cancel", "insert code"];
      d.addButtonListener("cancel", d.close);
      d.addButtonListener("insert code", Editor.listeners.uriWizardInsert);
      d.content = document.getElementById("wd-ed-uriwizard-dialog")!.innerHTML.replace(/-template/g, '');
      d.show();
    },
    pageLinkWizard: function (_event?: Event): void {
      Editor.currentPos = Editor.ranger!.getSelectionRange()[0];
      const d = new OZONE.dialogs.Dialog();
      d.title = ogettext("Page link wizard");
      d.buttons = ["cancel", "insert code"];
      d.addButtonListener("cancel", d.close);
      d.addButtonListener("insert code", Editor.listeners.pageLinkWizardInsert);
      d.content = document.getElementById("wd-ed-pagelinkwizard-dialog")!.innerHTML.replace(/-template/g, "");
      d.show();
      // attach the autocomplete thing
      const myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
      myDataSource.scriptQueryParam = "q";
      myDataSource.scriptQueryAppend = `s=${WIKIREQUEST.info.siteId}&module=PageLookupQModule&title=yes`;

      const myAutoComp = new YAHOO.widget.AutoComplete("wd-ed-pagelinkwizard-page", "autocomplete3432", myDataSource);
      myAutoComp.formatResult = (aResultItem: unknown, _sQuery: unknown) => {
        // @ts-expect-error Yahoo specific
        const title = aResultItem[1];
        // @ts-expect-error Yahoo specific
        const unixName = aResultItem[0];
        if (unixName != null) {
          return '<div >' + unixName + '</div><div style="font-size: 85%;">(' + title + ')</div>';
        } else {
          return "";
        }
      };
      myAutoComp.minQueryLength = 2;
      myAutoComp.queryDelay = 0.5;
      myAutoComp.forceSelection = false;
      myAutoComp.autoHighlight = false;
    },
    codeWizard: function (_event?: Event): void {
      Editor.currentPos = Editor.ranger!.getSelectionRange()[0];
      // open a dialog...
      const d = new OZONE.dialogs.Dialog();
      d.title = ogettext("Code block wizard");
      d.buttons = ["cancel", "insert code"];
      d.addButtonListener("cancel", d.close);
      d.addButtonListener("insert code", Editor.listeners.codeWizardInsert);
      d.content = document.getElementById("wd-ed-codewizard-dialog")!.innerHTML.replace(/-template/g, "");
      d.show();
    },
    erefWizard: function (_event?: Event): void {
      Editor.currentPos = Editor.ranger!.getSelectionRange()[0];
      // open a dialog...
      const d = new OZONE.dialogs.Dialog();
      d.title = ogettext("Equation reference wizard");
      d.buttons = ["cancel", "insert code"];
      d.addButtonListener("cancel", d.close);
      d.addButtonListener("insert code", Editor.erefWizard.insertCode);
      d.content = document.getElementById("wd-ed-erefwizard-dialog")!.innerHTML.replace(/-template/g, "");
      d.show();
      // now find all the equations...
      const text = (
        document.getElementById(Editor.editElementId) as HTMLTextAreaElement
      ).value;
      const refs = text.match(/^\[\[math\s([a-zA-Z0-9]+)\]\](\r?\n.*)*?\r?\n\[\[\/math\]\]/mg);
      if (refs == null || refs.length === 0) {
        const w = new OZONE.dialogs.ErrorDialog();
        w.content = "Sorry, no labelled equations found.";
        w.show();
        return;
      }
      let inn = '';
      if (refs.length === 0) {
        inn = "no equations with labels available";
      } else {
        inn = '<select id="wd-ed-erefwizard-ref">';
        for (let i = 0; i < refs.length; i++) {
          const elabel = refs[i].replace(/\[\[math\s(.+?)\]\](\r*\n.*)*/, '$1');
          const epreview = refs[i].replace(/\[\[math[^\]]*\]\]((?:\r?\n.*)*?)\n\[\[\/math\]\]/, "$1");
          document.getElementById("wd-ed-erefwizard-preview")!.innerHTML += '<div id="wd-ed-erefwizard-preview-' + elabel + '">' + epreview + '</div>';
          inn += '<option value="' + elabel + '">' + elabel + '</option>';
        }
        inn += '</select>';
      }
      document.getElementById("wd-ed-erefwizard-options")!.innerHTML = inn;
      OZONE.dialog.factory.boxcontainer().centerContent();
      Editor.erefWizard.changeRef();
      YAHOO.util.Event.addListener("wd-ed-erefwizard-ref", "change", Editor.erefWizard.changeRef);
    }
  },

  erefWizard: {
    changeRef: function (_event?: Event): void {
      const pdiv = document.getElementById("wd-ed-erefwizard-preview")!;
      const prevs = pdiv.children as HTMLCollectionOf<HTMLElement>;
      for (let i = 0; i < prevs.length; i++) {
        prevs[i].style.display = "none";
      }
      // @ts-expect-error Obviously
      document.getElementById("wd-ed-erefwizard-preview-" + document.getElementById("wd-ed-erefwizard-ref").value).style.display = "block";
    },
    insertCode: function (_event?: Event): void {
      const elabel = (
        document.getElementById("wd-ed-erefwizard-ref") as HTMLInputElement
      ).value;
      let out = `[[eref ${elabel}]]`;
      if (
        (
          document.getElementById("wd-ed-erefwizart-weq") as HTMLInputElement
        ).checked
      ) {
        out = 'Eq.(' + out + ')';
      }
      Editor.ranger!.setSelectionRange(Editor.currentPos, Editor.currentPos);
      Editor.utils.insertText(out);
      OZONE.dialog.cleanAll();
    }
  },

  imageWizard: {
    source: null as null | 'uri' | 'file',
    updateSourceBlock: function (_event?: Event): void {
      let source: null | 'uri' | 'file';
      document.getElementById("wd-ed-imagewizard-byuri")!.style.display = "none";
      document.getElementById("wd-ed-imagewizard-byfile")!.style.display = "none";
      document.getElementById("wd-ed-imagewizard-checkresult")!.innerHTML = "";

      if ((document.getElementById("342type1") as HTMLInputElement).checked) {
        source = "uri";
        document.getElementById("wd-ed-imagewizard-byuri")!.style.display = "block";
      } else if ((document.getElementById("342type2") as HTMLInputElement).checked) {
        source = "file";
        document.getElementById("wd-ed-imagewizard-byfile")!.style.display = "block";

        Editor.imageWizard.updateAttachements();
      } else {
        source = null;
      }
      Editor.imageWizard.source = source;
    },

    updateAttachements: function (): void {
      OZONE.ajax.requestModule(
        "Editor/ImageAttachedFileModule",
        { pageId: WIKIREQUEST.info.pageId! },
        Editor.imageWizard.updateAttachementsCallback
      );
    },
    updateAttachementsCallback: function (response: YahooResponse): void {
      document.getElementById("wd-ed-imagewizard-byfile-list")!.innerHTML = response.body;
      Editor.imageWizard.attachementSelect();
    },

    attachementSelect: function (_event?: Event): void {
      const el = document.getElementById("wd-ed-imagewizard-byfile-filename") as HTMLSelectElement;
      if (el) {
        const filename = el.value;
        const src = '/local--resized-images/' + WIKIREQUEST.info.requestPageName + '/' + filename + '/thumbnail.jpg';
        (
          document.getElementById("wd-ed-imagewizard-byfile-preview") as HTMLImageElement
        ).src = src;
      }
    },

    checkUriImage: function (_event?: Event): void {
      //  newwindow.titlebar=
      const input = (
        document.getElementById("wd-ed-imagewizard-uri") as HTMLInputElement
      ).value;
      const newwindow = window.open(
        "about:blank",
        "_blank",
        compressTight`
          location=no,menubar=no,titlebar=no,resizable=yes,scrollbars=yes,
          width=${screen.width * 0.5},
          height=${screen.height * 0.5},
          top=${screen.height * 0.25},
          left=${screen.width * 0.25}
        `
      )!;
      newwindow.document.write(
        compress`
          <html><head><title>Checking image...</title></head>
          <body><div style="text-align: center">
          <p>If you see the image below - that means the location of the image
          you have entered is ok.</p>
          <img id="check-image" src="${input}" alt="image not available!"/>
          <p><a href="javascript:;" onclick="window.close()">
          close this window
          </a></p></div></body></html>
        `
      );
      const ii = newwindow.document.getElementById("check-image");
      YAHOO.util.Event.addListener(ii, "load", Editor.imageWizard.checkUriImageResize, newwindow);
    },
    checkUriImageResize: function (_event: Event, win: Window): void {
      // resize the window
      // @ts-expect-error What is this?
      const width = Math.min(this.width + 200, screen.availWidth - 100);
      // @ts-expect-error What is this?
      const height = Math.min(this.height + 200, screen.availHeight - 100);
      const posleft = (screen.availWidth - width) * 0.5;
      const postop = (screen.availHeight - height) * 0.5;
      win.resizeTo(width, height);
      win.moveTo(posleft, postop);
    },

    insertCode: function (_event?: Event): void {
      const sourceType = Editor.imageWizard.source;
      let source;
      if (sourceType === "uri") {
        source = (
          document.getElementById("wd-ed-imagewizard-uri") as HTMLInputElement
        ).value;
      } else if (sourceType === "file") {
        source = (
          document.getElementById("wd-ed-imagewizard-byfile-filename") as HTMLSelectElement
        ).value;
      }

      // check if size
      let size = '';
      const el = document.getElementById("wd-ed-imagewizard-size") as HTMLSelectElement;
      if (el) {
        size = el.value;
      }

      if (size !== '') {
        size = ' size="' + size + '"';
      }

      const position = (
        document.getElementById("wd-ed-imagewizard-position") as HTMLSelectElement
      ).value.replace(/l/, '<').replace(/r/, '>').replace(/c/, '=');
      const code = '[[' + position + 'image ' + source + size + ']]';
      Editor.ranger!.setSelectionRange(Editor.currentPos, Editor.currentPos);
      Editor.utils.insertText(code);
      OZONE.dialog.cleanAll();
    }
  },
  /**
   * Mainly wizard button listeners...
   */
  listeners: {
    tableWizardInsert: function (_event?: Event): void {
      const rows = parseInt((
        document.getElementById("wd-ed-tablewizard-rows") as HTMLInputElement
      ).value);
      const columns = parseInt((
        document.getElementById("wd-ed-tablewizard-columns") as HTMLInputElement
      ).value);
      const headers = (
        document.getElementById("wd-ed-tablewizard-headers")as HTMLInputElement
      ).checked;

      // prepare code to be inserted
      let out = '';
      for (let i = 0; i < rows; i++) {
        out += INSERT_NEWLINE + '||';
        for (let j = 0; j < columns; j++) {
          if (i === 0 && headers) {
            out += "~ header ||";
          } else {
            out += " cell-content ||";
          }
        }
      }

      // insert it!
      Editor.ranger!.setSelectionRange(Editor.currentPos, Editor.currentPos);
      Editor.utils.insertText(
        out,
        Editor.utils.endWithAtLeast1NewLine,
        Editor.utils.startWithAtLeast2NewLine
      );
      OZONE.dialog.cleanAll();
    },
    uriWizardInsert: function (_event?: Event): void {
      const uri = (
        document.getElementById("wd-ed-uriwizard-uri") as HTMLInputElement
      ).value;
      const anchor = (
        document.getElementById("wd-ed-uriwizard-anchor") as HTMLInputElement
      ).value;
      const newwindow = (
        document.getElementById("wd-ed-uriwizard-newwindow") as HTMLInputElement
      ).checked;

      let out = '';
      if (anchor == null || anchor === '') {
        if (newwindow) {
          out += '*';
        }
        out += uri;
      } else {
        out = '[';
        if (newwindow) {
          out += '*';
        }
        out += uri + ' ' + anchor + ']';
      }
      Editor.ranger!.setSelectionRange(Editor.currentPos, Editor.currentPos);
      Editor.utils.insertText(out);
      OZONE.dialog.cleanAll();
    },
    pageLinkWizardInsert: function (_event?: Event): void {
      const pageName = (
        document.getElementById("wd-ed-pagelinkwizard-page") as HTMLInputElement
      ).value;
      const anchor = (
        document.getElementById("wd-ed-pagelinkwizard-anchor") as HTMLInputElement
      ).value;

      let out = '[[[' + pageName;
      if (anchor != null && anchor !== '') {
        out += ' |' + anchor;
      }
      out += ']]]';
      Editor.ranger!.setSelectionRange(Editor.currentPos, Editor.currentPos);
      Editor.utils.insertText(out);
      OZONE.dialog.cleanAll();
    },

    codeWizardInsert: function (_event?: Event): void {
      const type = (
        document.getElementById("wd-ed-codewizard-type") as HTMLInputElement
      ).value;
      let openTag = '[[code';
      if (type !== '') {
        openTag += ' type="' + type + '"';
      }
      openTag += "]]" + INSERT_NEWLINE;
      const closeTag = INSERT_NEWLINE + "[[/code]]";

      Editor.ranger!.setSelectionRange(Editor.currentPos, Editor.currentPos);
      Editor.utils.insertTags(
        openTag, closeTag, "insert the code here",
        Editor.utils.trimSelection,
        Editor.utils.endWithAtLeast2NewLine,
        Editor.utils.startWithAtLeast2NewLine
      );
      OZONE.dialog.cleanAll();
    }
  },

  keyboardListener: function (event: KeyboardEvent): void {
    Editor.lastKeyCode = null;
    const keyCode = YAHOO.util.Event.getCharCode(event);
    Editor.lastKeyCode = keyCode;
    //    // trigger codeAssist
    let key = '';
    if (event.ctrlKey) key += "ctrl+";
    if (event.altKey) key += "alt+";
    key += String.fromCharCode(keyCode);
    if (document.getElementById("editdebug")) {
      document.getElementById("editdebug")!.innerHTML = keyCode;
    }
    const keys = {
      "ctrl+b": Editor.buttons.bold,
      "ctrl+i": Editor.buttons.italic,
      "ctrl+u": Editor.buttons.underline
    };
    const keyCodes = {
      9: (event: Event) => {
        Editor.utils.insertText("\t");
        YAHOO.util.Event.stopEvent(event);
      }
    };
    let listener: (event: Event) => void;
    if (key in keys) {
      listener = keys[key as keyof typeof keys];
    } else if (keyCode in keyCodes) {
      listener = keyCodes[keyCode as keyof typeof keyCodes];
    } else {
      return;
    }
    if (listener) {
      YAHOO.util.Event.preventDefault(event);
      listener(event);
    }
  },
  codeAssist: {
    listener: function (_event?: Event): void {
      const keyCode = Editor.lastKeyCode;
      if (keyCode !== 13) {
        return;
      }

      // need to insert the "\n" manually here and stop event propagation
      // perform a number of checks if one should insert anything interesting
      // check for a list item first

      Editor.utils.insertText("", Editor.codeAssist.rules.listEnd);//,
      Editor.utils.insertText("", Editor.codeAssist.rules.list);//,
      Editor.utils.insertText("", Editor.codeAssist.rules.listNested);//,

      Editor.codeAssist.rules.completeBlock();
      Editor.utils.insertText("", Editor.codeAssist.rules.definitionList);//,
      Editor.utils.insertText("", Editor.codeAssist.rules.keepIndent);
      Editor.utils.insertText("", Editor.codeAssist.rules.indentEnd);
    },
    rules: {

      list: function (text: string): string {
        /* check if previous line has anything to do with lists, i.e.
         * 1. check if previous line starts with #|*
         */
        text = text.replace(/(\r?\n([*#])\s.*?\r?\n)$/, "$1$2 ");
        return text;
      },
      definitionList: function (text: string): string {
        text = text.replace(/(\r?\n:\s.+?\s:.*\r?\n)$/, "$1: ");
        return text;
      },
      listNested: function (text: string): string {
        /* this is different from list because requires one more line before
         * check if previous line has anything to do with lists, i.e.
         * 1. check if previous line starts with #|*
         *
         */
        text = text.replace(/(\r?\n *[*#]\s.+\r?\n( *)([*#])\s.*?\r?\n)$/, "$1$2$3 ");
        return text;
      },
      listEnd: function (text: string): string {
        /* after "double enter" remove list markup */
        text = text.replace(/(\r?\n\s*[*#:]\s.*?\r?\n)\s*[*#:]\s\r?\n$/, "$1" + INSERT_NEWLINE);
        return text;
      },
      keepIndent: function (text: string): string {
        /* keeps the identation from the previous line */
        text = text.replace(/(\r?\n(\t+).+\r?\n)$/, "$1$2");
        return text;
      },
      indentEnd: function (text: string): string {
        /* keeps the identation from the previous line */
        text = text.replace(/(\r?\n(\t+)\r?\n)$/, INSERT_NEWLINE + INSERT_NEWLINE);
        return text;
      },
      /**
       * Checks if the previous line contains any block start mark and stores
       * the name of the block in a variable
       */
      completeBlock: function (): void {
        const field = document.getElementById(Editor.editElementId) as HTMLTextAreaElement;
        const scrollTop = field.scrollTop;
        const ranger = Editor.ranger!;
        const range = ranger.getSelectionRange();
        let before = field.value.substring(0, range[1]);
        const after = field.value.substring(range[1], field.value.length);
        const beforeOrigLength = before.length;
        before = before.replace(/(\[\[(div|code|embedvideo|math|embed)(?:\s[^\]]*?)?\]\]\r?\n)$/, "$1" + INSERT_NEWLINE + "[[/$2]]");
        field.value = before + after;
        const cursorPos = beforeOrigLength;
        ranger.setSelectionRange(cursorPos, cursorPos);
        field.scrollTop = scrollTop;
      }
    }
  },

  utils: {
    insertTags: function (
      openTag: string,
      closeTag: string,
      sampleText: string,
      processSelection?: null | ((text: string) => string),
      processBefore?: null | ((text: string) => string),
      processAfter?: null | ((text: string) => string),
      dontSelectSampleText?: boolean
    ): void {
      const myField = document.getElementById(Editor.editElementId) as HTMLTextAreaElement;
      myField.focus();
      const ranger = Editor.ranger!;
      ranger.trimSelection();
      const range = ranger.getSelectionRange();

      const scrollTop = myField.scrollTop;

      let beforeText = myField.value.substring(0, range[0]);
      if (processBefore) {
        beforeText = processBefore(beforeText);
      }
      let afterText = myField.value.substring(range[1], myField.value.length);
      if (processAfter) {
        afterText = processAfter(afterText);
      }

      if (range[0] !== range[1]) {
        let selectionText = myField.value.substring(range[0], range[1]);
        if (processSelection) {
          selectionText = processSelection(selectionText);
        }

        myField.value = beforeText +
          openTag +
          selectionText +
          closeTag +
          afterText;
        const cursorPos = myField.value.length - afterText.length;
        ranger.setSelectionRange(cursorPos, cursorPos);
      } else {
        myField.value = beforeText +
          openTag +
          sampleText +
          closeTag +
          afterText;
        if (!dontSelectSampleText) {
          const startPos = beforeText.length + openTag.length;
          const endPos = startPos + sampleText.length;
          ranger.setSelectionRange(startPos, endPos);
        } else {
          // just position the cursor after the text
          const cursorPos = myField.value.length - afterText.length;
          ranger.setSelectionRange(cursorPos, cursorPos);
        }
      }
      myField.focus();
      myField.scrollTop = scrollTop;
    },

    insertText: function (
      text: string,
      processBefore?: null | ((text: string) => string),
      processAfter?: null | ((text: string) => string)
    ): void {
      Editor.utils.insertTags(
        '', '', text,
        null, processBefore, processAfter, true
      );
    },

    trimSelection: function (text: string): string {
      return text.replace(/^\s+/, '').replace(/\s+$/, '');
    },
    endWithNewLine: function (text: string): string {
      /**
       * Checks if a string ends with a newline and of no, adds it.
       */
      return text.replace(/[\s\r\n]+$/, '') + INSERT_NEWLINE;
    },
    endWithAtLeast1NewLine: function (text: string): string {
      return text.replace(/\r?\n$/, '') + INSERT_NEWLINE;
    },
    startWithNewLine: function (text: string): string {
      return INSERT_NEWLINE + text.replace(/^[\s\r\n]+/, '');
    },
    startWithAtLeast1NewLine: function (text: string): string {
      if (text.length === 0) {
        return text;
      }
      return INSERT_NEWLINE + text.replace(/^\r?\n/, '');
    },

    startWithAtLeast2NewLine: function (text: string): string {
      if (text.length === 0) {
        return text;
      }
      return INSERT_NEWLINE + INSERT_NEWLINE + text.replace(/^\r?\n(\s*\r?\n)?/, '');
    },
    endWithAtLeast2NewLine: function (text: string): string {
      if (text.length === 0) {
        return text;
      }
      return text.replace(/(\r?\n\s*)?\r?\n$/, '') + INSERT_NEWLINE + INSERT_NEWLINE;
    },

    endWith2NewLine: function (text: string): string {
      if (text.length === 0) {
        return text;
      }
      return text.replace(/[\s\r\n]+$/, '') + INSERT_NEWLINE + INSERT_NEWLINE;
    },

    startWith2NewLine: function (text: string): string {
      return INSERT_NEWLINE + INSERT_NEWLINE + text.replace(/^[\s\r\n]+/, '');
    },

    processQuoteText: function (text: string): string {
      text = text.replace(/^\s+/, '').replace(/\s+$/, '');
      text = text.replace(/\r?\n/g, INSERT_NEWLINE + "> ");
      return text;
    },
    processNumberedList: function (text: string): string {
      text = text.replace(/^\s+/, '').replace(/\s+$/, '');
      text = text.replace(/\r?\n/g, INSERT_NEWLINE + "# ");
      return text;
    },
    processBulletedList: function (text: string): string {
      text = text.replace(/^\s+/, '').replace(/\s+$/, '');
      text = text.replace(/\r?\n/g, INSERT_NEWLINE + "* ");
      return text;
    },
    increaseListIndent: function (text: string): string {
      // check if not "overnested"
      if (text.match(/\r?\n(\s*)[*#].*\r?\n(\1)\s+[*#].*$/)) {
        return text;
      }
      return text.replace(/(\r?\n\s*[*#].*)(\r?\n\s*)([*#].*)$/, "$1$2 $3");
    },
    decreaseListIndent: function (text: string): string {
      return text.replace(/(\r?\n\s*) ([*#].*)$/, "$1$2");
    }
  }
};
