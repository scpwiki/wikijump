import Wikijump from "@/javascript/Wikijump";
import { editMode } from "@/javascript/Wikijump/page";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import { Wikirequest } from "wikirequest";
declare const WIKIREQUEST: Wikirequest;
declare const YAHOO: any;
declare type YahooResponse = any;

export const PageEditModule = {
  vars: {
    editMode: 'page' as typeof editMode, // the default mode
    stopCounterFlag: false,
    inputFlag: false, // changed to true by any input
    lastInput: (new Date()).getTime(), // last input.
    savedSource: '', // source saved on server
    counterStart: 0,
    lockLastUpdated: 0,
    counterEmergency: false,
    lockExpire: 0,
    timerId: 0,
  },
  listeners: {
    cancel: function (_event?: Event | null): void {
      YAHOO.util.Event.removeListener(window, "beforeunload", PageEditModule.listeners.leaveConfirm);
      YAHOO.util.Event.removeListener(window, "unload", PageEditModule.listeners.leavePage);
      // XXX This element does not appear to be defined anywhere
      if (document.getElementById('wikijump-disable-locks-flag')) {
        window.location.href = `/${WIKIREQUEST.info.requestPageName}`;
        return;
      }
      const params: RequestModuleParameters = {
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret,
        action: "WikiPageAction",
        event: "removePageEditLock"
      };
      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.cancel);
    },
    preview: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("edit-page-form"),
        mode: PageEditModule.vars.editMode,
        revision_id: Wikijump.page.vars.editlock.revisionId,
        page_unix_name: WIKIREQUEST.info.requestPageName,
      };
      if (WIKIREQUEST.info.pageId) {
        params.pageId = WIKIREQUEST.info.pageId;
      }

      OZONE.ajax.requestModule("edit/PagePreviewModule", params, PageEditModule.callbacks.preview);
    },
    save: function (_event?: Event | null): void {
      const t2 = new OZONE.dialogs.WaitBox(); // global??? pheeee...
      t2.content = "Saving page...";
      t2.show();

      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("edit-page-form"),
        action: 'WikiPageAction',
        event: 'savePage',
        mode: PageEditModule.vars.editMode,
        wiki_page: WIKIREQUEST.info.requestPageName,
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret,
        revision_id: Wikijump.page.vars.editlock.revisionId,
      };
      if (WIKIREQUEST.info.pageId) {
        params.page_id = WIKIREQUEST.info.pageId;
      }
      if (PageEditModule.vars.editMode === 'section') {
        params.range_start = Wikijump.page.vars.editlock.rangeStart;
        params.range_end = Wikijump.page.vars.editlock.rangeEnd;
      }

      /* Handle tags. */
      try {
        const path = location.href.toString();
        const zz = /\/tags\/([^/]+)/.exec(path);
        if (zz) {
          // set the title
          params.tags = decodeURIComponent(zz[1]);
        }
      } catch (error) {
        void error;
      }
      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.save);
    },
    saveAndContinue: function (_event?: Event | null): void {
      const t2 = new OZONE.dialogs.WaitBox();
      t2.content = "Saving page...";
      t2.show();

      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("edit-page-form"),
        action: 'WikiPageAction',
        event: 'savePage',
        mode: PageEditModule.vars.editMode,
        wiki_page: WIKIREQUEST.info.requestPageName,
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret,
        revision_id: Wikijump.page.vars.editlock.revisionId,
        and_continue: "yes",
      };
      if (WIKIREQUEST.info.pageId) {
        params.page_id = WIKIREQUEST.info.pageId;
      }
      if (PageEditModule.vars.editMode == 'section') {
        params.range_start = Wikijump.page.vars.editlock.rangeStart;
        params.range_end = Wikijump.page.vars.editlock.rangeEnd;
      }

      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.saveAndContinue);
    },
    changeInput: function (_event?: Event | null): void {
      PageEditModule.vars.inputFlag = true;

      PageEditModule.vars.lastInput = (new Date()).getTime();
      PageEditModule.utils.timerSetTimeLeft(15 * 60);
    },
    leaveConfirm: function (event: Event): void {
      if (PageEditModule.utils.sourceChanged()) {
        //@ts-expect-error returnValue expects a bool - there should be some
        //other way of getting this message to the user
        event.returnValue = 'If you leave this page, all the unsaved changes will be lost.';
      }
    },
    leavePage: function (_event?: Event | null): void {
      // release lock
      const params: RequestModuleParameters = {
        action: "WikiPageAction",
        event: "removePageEditLock",
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret
      };
      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.forcePageEditLockRemove);

      /* DO WE NEED THIS INFO????
         if(PageEditModule.utils.sourceChanged()){
         alert("You have closed or left the window while editing a page.\n" +
         "If you have made any changes without saving them they are lost now.\n" +
         "The page edit lock has been removed.");
         } */
      // XXX ^ *Do* we need this info?
    },
    forcePageEditLockRemove: function (_event?: Event | null): void {
      Wikijump.page.vars.forceLockFlag = true;
      OZONE.dialog.cleanAll();
      Wikijump.page.listeners.editClick();
    },
    forceLockIntercept: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: 'WikiPageAction',
        event: 'forceLockIntercept',
        mode: PageEditModule.vars.editMode,
        wiki_page: WIKIREQUEST.info.requestPageName,
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret,
        revision_id: Wikijump.page.vars.editlock.revisionId,
      };
      if (WIKIREQUEST.info.pageId) {
        params.page_id = WIKIREQUEST.info.pageId;
      }
      if (PageEditModule.vars.editMode === 'section') {
        params.range_start = Wikijump.page.vars.editlock.rangeStart;
        params.range_end = Wikijump.page.vars.editlock.rangeEnd;
      }

      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.forceLockIntercept);
    },
    recreateExpiredLock: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        action: 'WikiPageAction',
        event: 'recreateExpiredLock',
        mode: PageEditModule.vars.editMode,
        wiki_page: WIKIREQUEST.info.requestPageName,
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret,
        revision_id: Wikijump.page.vars.editlock.revisionId,
        since_last_input: 0, // seconds since last input
      };
      if (WIKIREQUEST.info.pageId) {
        params.page_id = WIKIREQUEST.info.pageId;
      }
      if (PageEditModule.vars.editMode == 'section') {
        params.range_start = Wikijump.page.vars.editlock.rangeStart;
        params.range_end = Wikijump.page.vars.editlock.rangeEnd;
      }

      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.recreateExpiredLock);
    },
    templateChange: function (_event?: Event | null): void {
      if (!document.getElementById("page-templates")!) { return; }
      const templateId = (<HTMLSelectElement>document.getElementById("page-templates")!).value;
      let change = true;
      if (PageEditModule.utils.sourceChanged()) {
        change = confirm("It seems you have already changed the page.\n" +
                         "Changing the initial template now will reset the edited page.\n" +
                         "Do you want to change the initial content?");
      }
      if (change) {
        if (templateId == null || templateId == "") {
          (<HTMLTextAreaElement>document.getElementById("edit-page-textarea")!).value = '';
        } else {
          const params: RequestModuleParameters = {
            page_id: templateId
          };
          OZONE.ajax.requestModule("edit/TemplateSourceModule", params, PageEditModule.callbacks.templateChange);
        }
      } else {
        (<HTMLSelectElement>document.getElementById("page-templates")!).value = templateId;
      }
    },
    viewDiff: function (_event?: Event | null): void {
      const params: RequestModuleParameters = {
        ...OZONE.utils.formToArray("edit-page-form"),
        mode: PageEditModule.vars.editMode,
        revision_id: Wikijump.page.vars.editlock.revisionId,
      };
      if (PageEditModule.vars.editMode == 'section') {
        params.range_start = Wikijump.page.vars.editlock.rangeStart;
        params.range_end = Wikijump.page.vars.editlock.rangeEnd;
      }
      OZONE.ajax.requestModule("edit/PageEditDiffModule", params, PageEditModule.callbacks.viewDiff);
    },
    confirmExpiration: function (_event?: Event | null): void {
      PageEditModule.utils.deactivateAll();
      OZONE.dialog.cleanAll();
    },
    closeDiffView: function (_event?: Event | null): void {
      OZONE.visuals.scrollTo('action-area');
      setTimeout(() => document.getElementById("view-diff-div")!.innerHTML="", 250);
    }
  },
  callbacks: {
    preview: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const message = document.getElementById("preview-message")!.innerHTML;
      OZONE.utils.setInnerHTMLContent("action-area-top", message);

      if (PageEditModule.vars.editMode == 'section') {
        OZONE.utils.setInnerHTMLContent("edit-section-content", response.body.replace(/id="/g, 'id="prev06-'));
        YAHOO.util.Dom.setY("action-area-top", YAHOO.util.Dom.getY("edit-section-content"));
        OZONE.visuals.scrollTo("edit-section-content");
      }
      if (PageEditModule.vars.editMode == 'page') {
        const title = response.title;
        OZONE.utils.setInnerHTMLContent("page-title", title);
        OZONE.visuals.scrollTo("container");
        document.getElementById("page-content")!.innerHTML = response.body;
        Wikijump.page.fixers.fixEmails(document.getElementById("page-content")!);
      }
      // put some notice that this is a preview only!

      if (PageEditModule.vars.editMode == 'append') {
        let aDiv = document.getElementById("append-preview-div")!;
        if (!aDiv) {
          aDiv = document.createElement('div');
          aDiv.id = "append-preview-div";
          document.getElementById("page-content")!.appendChild(aDiv);
        }

        aDiv.innerHTML = response.body.replace(/id="/g, 'id="prev06-');

        Wikijump.page.fixers.fixEmails(document.getElementById("append-preview-div")!);
        OZONE.visuals.scrollTo("append-preview-div");
        // move the message box
        YAHOO.util.Dom.setY("action-area-top", YAHOO.util.Dom.getY("append-preview-div"));
      }

      PageEditModule.utils.stripAnchors("page-content", "action-area");
    },
    viewDiff: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById("view-diff-div")!.innerHTML = response.body;
      OZONE.visuals.scrollTo("view-diff-div");
    },
    save: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      // check for errors?
      if (response.noLockError) {
        // non recoverable. not saved.
        PageEditModule.utils.timerStop();

        const w = new OZONE.dialogs.ErrorDialog();
        w.content = response.body;
        w.show();

        if (response.nonrecoverable == true) {
          PageEditModule.utils.deactivateAll();
        }

        return;
      }
      PageEditModule.utils.timerStop();

      setTimeout(() => OZONE.dialog.factory.boxcontainer().hide({smooth: true}), 400);
      setTimeout(() => {
        const t2 = new OZONE.dialogs.SuccessBox();
        t2.timeout = 10000;
        t2.content = "Page saved!";
        t2.show();
      }, 600);
      let newUnixName = WIKIREQUEST.info.requestPageName;
      if (response.pageUnixName) {
        newUnixName = response.pageUnixName;
      }
      setTimeout(() => { window.location.href = `/${newUnixName}`; }, 1500);

      YAHOO.util.Event.removeListener(window, "beforeunload", PageEditModule.listeners.leaveConfirm);
      YAHOO.util.Event.removeListener(window, "unload", PageEditModule.listeners.leavePage);
    },
    saveAndContinue: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      if (response.noLockError) {
        PageEditModule.utils.timerStop();
        const cont = OZONE.dialog.factory.boxcontainer();
        cont.setContent(response.body);
        cont.showContent();
        if (response.nonrecoverable == true) {
          PageEditModule.utils.deactivateAll();
        }
        return;
      }
      setTimeout(() => OZONE.dialog.factory.boxcontainer().hide({smooth: true}), 400);
      setTimeout(() => {
        const t2 = new OZONE.dialogs.SuccessBox();
        t2.content="Page saved!";
        t2.show();
      }, 600);
      setTimeout(() => OZONE.dialog.cleanAll(), 2000);
      PageEditModule.utils.updateSavedSource();
      Wikijump.page.vars.editlock.revisionId = response.revisionId;
    },
    cancel: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      window.location.href = `/${WIKIREQUEST.info.requestPageName}`;
    },
    forcePageEditLockRemove: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      Wikijump.page.listeners.editClick();
    },
    forceLockIntercept: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      if (response.error) {
        alert('Unexpected error');
        return;
      }
      if (response.nonrecoverable == true) {
        const cont = OZONE.dialog.factory.boxcontainer();
        cont.setContent(response.body);
        cont.showContent();
      }
      PageEditModule.utils.timerSetTimeLeft(response.timeLeft);
      PageEditModule.utils.timerStart();
      Wikijump.page.vars.editlock.id = response.lock_id;
      Wikijump.page.vars.editlock.secret = response.lock_secret;
      const t2 = new OZONE.dialogs.SuccessBox(); // global??? pheeee...
      t2.content = "Lock successfully acquired";
      t2.show();
    },
    updateLock: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      // check for errors?

      if (response.noLockError) {
        OZONE.dialog.factory.shader().show();
        const cont = OZONE.dialog.factory.boxcontainer();
        cont.setContent(response.body);
        cont.showContent();

        if (response.nonrecoverable == true) {
          PageEditModule.utils.deactivateAll();
        }
        PageEditModule.utils.timerStop();
        YAHOO.util.Event.removeListener(window, "beforeunload", PageEditModule.listeners.leaveConfirm);
        YAHOO.util.Event.removeListener(window, "unload", PageEditModule.listeners.leavePage);
        return;
      }

      if (response.lockRecreated) {
        Wikijump.page.vars.editlock.id = response.lockId;
        Wikijump.page.vars.editlock.secret = response.lockSecret;
      }
      PageEditModule.utils.timerSetTimeLeft(response.timeLeft);
    },
    lockExpired: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.dialog.factory.shader().show();
      const cont = OZONE.dialog.factory.boxcontainer();
      cont.setContent(response.body);
      cont.showContent();
    },
    recreateExpiredLock: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      if (!response.lockRecreated) {
        OZONE.dialog.factory.shader().show();
        const cont = OZONE.dialog.factory.boxcontainer();
        cont.setContent(response.body);
        cont.showContent();
      } else {
        Wikijump.page.vars.editlock.id = response.lockId;
        Wikijump.page.vars.editlock.secret = response.lockSecret;
        PageEditModule.utils.timerSetTimeLeft(response.timeLeft);
        PageEditModule.utils.timerStart();
        PageEditModule.vars.lastInput = (new Date()).getTime();
        const t2 = new OZONE.dialogs.SuccessBox(); // global??? pheeee...
        t2.content = "Lock succesfully acquired.";
        t2.show();
      }
    },
    templateChange: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      if (response.body != null && response.body != "") {
        (<HTMLTextAreaElement>document.getElementById("edit-page-textarea")!).value = response.body;
      }
    }
  },
  utils: {
    sourceChanged: function (): boolean {
      const a = OZONE.utils.formToArray("edit-page-form");
      return (PageEditModule.vars.savedSource !== a.source);
    },
    updateSavedSource: function (): void {
      const a = OZONE.utils.formToArray("edit-page-form");
      PageEditModule.vars.savedSource = a.source;
    },
    stripAnchors: function (elementId: string, excludeElementId?: string): void {
      const el = document.getElementById(elementId)!;
      let excludeElement;
      if (excludeElementId) {
        excludeElement = document.getElementById(excludeElementId)!;
      }
      if (el) {
        const anchors = el.getElementsByTagName("a");

        for (let i = 0; i < anchors.length; i++) {
          if (excludeElement == null || !YAHOO.util.Dom.isAncestor(excludeElement, anchors[i])) {
            anchors[i].href = "javascript:;";
            anchors[i].onclick = null;
            anchors[i].target = "_self";
            YAHOO.util.Event.purgeElement(anchors[i]);
            YAHOO.util.Event.addListener(anchors[i], "click", PageEditModule.utils.leavePageWarning);
          }
        }
      }
    },
    /**
     * Replaces anchors with dumb anchors in edit mode
     */
    stripAnchorsAll: function (): void {
      PageEditModule.utils.stripAnchors("html-body", "action-area");
    },
    leavePageWarning: function (): void {
      alert("Oooops... You should not leave the page while editing it.\nTo abort editing please use the \"cancel\" button below the edit area.");
    },
    updateActiveButtons: function (): void {
      const el = (<HTMLInputElement>document.getElementById("edit-save-continue-button")!);
      if (el) {
        el.disabled = false;
        YAHOO.util.Dom.removeClass(el, "disabled");
      }
      (<HTMLInputElement>document.getElementById("edit-save-button")!).disabled = false;
      YAHOO.util.Dom.removeClass(document.getElementById("edit-save-button")!, "disabled");
    },
    deactivateAll: function (): void {
      // deactivates all the buttons, event.g. when lock is intercepted
      const el = (<HTMLInputElement>document.getElementById("edit-save-continue-button")!);
      if (el) {
        el.disabled = true;
      }
      (<HTMLInputElement>document.getElementById("edit-save-button")!).disabled = true;
      document.getElementById("lock-info")!.style.display = "none";
      YAHOO.util.Event.removeListener("edit-page-form", "keypress", PageEditModule.listeners.changeInput);
      YAHOO.util.Event.removeListener("edit-page-textarea", "change", PageEditModule.listeners.changeInput);
      YAHOO.util.Event.removeListener(window, "beforeunload", PageEditModule.listeners.leaveConfirm);
      YAHOO.util.Event.removeListener(window, "unload", PageEditModule.listeners.leavePage);
      PageEditModule.vars.stopCounterFlag = true;
    },
    startLockCounter: function (): void {
      PageEditModule.vars.counterStart = (new Date()).getTime();
      PageEditModule.vars.lockLastUpdated = (new Date()).getTime();
      PageEditModule.utils.updateLockCounter();
      PageEditModule.vars.counterEmergency = false;
    },
    updateLockCounter: function (): void {
      let sec = (new Date()).getTime() - PageEditModule.vars.counterStart;
      sec = Math.round(15 * 60 - sec * 0.001);
      OZONE.utils.setInnerHTMLContent("lock-timer", sec.toString());
      if (sec < 120 && PageEditModule.vars.counterEmergency == false) {
        document.getElementById("lock-timer")!.style.color = "red";
        document.getElementById("lock-timer")!.style.textDecoration = "blink";
        PageEditModule.vars.counterEmergency = true;
      }
      setTimeout(() => PageEditModule.utils.updateLockCounter(), 1000);
    },
    timerSetTimeLeft: function (timeLeft: number): void {
      PageEditModule.vars.lockExpire = (new Date()).getTime() + timeLeft * 1000;
    },
    timerTick: function (): void {
      let secLeft = PageEditModule.vars.lockExpire - (new Date()).getTime();
      secLeft = Math.round(secLeft * 0.001);
      document.getElementById("lock-timer")!.innerHTML = secLeft.toString();

      if (secLeft <= 0) {
        PageEditModule.utils.lockExpired();
        return;
      }

      const sinceLastUpdate = (new Date()).getTime() - PageEditModule.vars.lockLastUpdated;
      if (sinceLastUpdate * 0.001 >= 60 || (secLeft < 60 && PageEditModule.vars.inputFlag)) {
        PageEditModule.vars.inputFlag = false;
        PageEditModule.vars.lockLastUpdated = (new Date()).getTime();
        PageEditModule.utils.updateLock();
      }

      // do some action if conditions....
    },
    timerStart: function (): void {
      if (document.getElementById('wikijump-disable-locks-flag')!) { return; }
      PageEditModule.vars.timerId = setInterval(PageEditModule.utils.timerTick, 1000);
    },
    timerStop: function (): void {
      if (document.getElementById('wikijump-disable-locks-flag')!) { return; }
      clearInterval(PageEditModule.vars.timerId);
    },
    /**
     * Send a request to a server to update lock.
     */
    updateLock: function (): void {
      if (document.getElementById('wikijump-disable-locks-flag')!) { return; }
      const secSinceLastInput = Math.round(((new Date()).getTime() - PageEditModule.vars.lastInput) * 0.001);
      const params: RequestModuleParameters = {
        action: 'WikiPageAction',
        event: 'updateLock',
        mode: PageEditModule.vars.editMode,
        wiki_page: WIKIREQUEST.info.requestPageName,
        lock_id: Wikijump.page.vars.editlock.id,
        lock_secret: Wikijump.page.vars.editlock.secret,
        revision_id: Wikijump.page.vars.editlock.revisionId,
        since_last_input: secSinceLastInput
      };
      if (WIKIREQUEST.info.pageId) {
        params.page_id = WIKIREQUEST.info.pageId;
      }
      if (PageEditModule.vars.editMode == 'section') {
        params.range_start = Wikijump.page.vars.editlock.rangeStart;
        params.range_end = Wikijump.page.vars.editlock.rangeEnd;
      }

      OZONE.ajax.requestModule("Empty", params, PageEditModule.callbacks.updateLock);
    },
    lockExpired: function (): void {
      PageEditModule.utils.timerStop();

      OZONE.ajax.requestModule("edit/LockExpiredWinModule", {}, PageEditModule.callbacks.lockExpired);
    }
  },
  init: function (): void {
    if (Wikijump.page.vars.locked == true) {
      OZONE.utils.formatDates();
    } else {
      PageEditModule.vars.editMode = editMode;

      /* attach listeners */

      YAHOO.util.Event.addListener("update-lock", "click", PageEditModule.utils.updateLock);

      YAHOO.util.Event.addListener("edit-page-form", "keypress", PageEditModule.listeners.changeInput);
      YAHOO.util.Event.addListener("edit-page-textarea", "keydown", PageEditModule.listeners.changeInput);
      //

      YAHOO.util.Event.addListener(window, "beforeunload", PageEditModule.listeners.leaveConfirm);
      YAHOO.util.Event.addListener(window, "unload", PageEditModule.listeners.leavePage);

      PageEditModule.utils.stripAnchorsAll();
      PageEditModule.utils.updateSavedSource();
      PageEditModule.utils.updateActiveButtons();
      const path = window.location.pathname;
      const zz = /^\/[a-z0-9\-:]+\/edit\/true\/t\/([0-9]+)/.exec(path);
      if (zz) {
        // force use the template
        const templateId = zz[1];
        (<HTMLSelectElement>document.getElementById("page-templates")!).value = templateId;
      }

      try {
        const zz = /\/title\/([^/]+)/.exec(path);
        if (zz) {
          // set the title
          (<HTMLInputElement>document.getElementById("edit-page-title")!).value = decodeURIComponent(zz[1]);
        }
      } catch (error) {
        void error;
      }

      if (!WIKIREQUEST.info.pageId) {
        // new page - init templates!
        PageEditModule.listeners.templateChange(null);
      }

      PageEditModule.utils.timerSetTimeLeft(60 * 15);
      PageEditModule.vars.lockLastUpdated = (new Date().getTime());
      PageEditModule.utils.timerStart();

      const form = document.getElementById("edit-page-form")!.classList.contains("edit-with-form");
      if (!form) {
        Wikijump.Editor.init("edit-page-textarea", "wd-editor-toolbar-panel");
      } else {
        // No action to take - form editing does not need initalising
      }

      new OZONE.forms.lengthLimiter("edit-page-comments", "comments-charleft", 200);
      OZONE.dialog.cleanAll();

      // clear all visible hovers
      OZONE.dialog.hovertip.hideAll();

      // prevent backspace from going back
      YAHOO.util.Event.addListener(window, 'keypress', function (_event?: Event | null): void {
        const kc = YAHOO.util.Event.getCharCode(event);
        if (kc == 8) {
          const t = YAHOO.util.Event.getTarget(event, true);
          if (t.tagName.toLowerCase() != 'input' && t.tagName.toLowerCase() != 'textarea') {
            YAHOO.util.Event.stopEvent(event);
          }
        }
      });

      // handle ctrl+s
      // @ts-expect-error Yahoo
      const ctrls = new YAHOO.util.KeyListener(document, { keys: 83, ctrl: true }, function (type, event): void {
        event = event[1];
        PageEditModule.listeners.save(event);
        YAHOO.util.Event.stopEvent(event);
      });
      ctrls.enable();
      if (form) {
        // focus first field?
      } else {
        document.getElementById("edit-page-textarea")!.focus();
      }
    }
  }
};

// WHY??? ;-)
setTimeout(() => PageEditModule.init(), 10);
