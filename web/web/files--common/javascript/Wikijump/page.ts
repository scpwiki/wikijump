import Wikijump from ".";
import OZONE from "@/javascript/OZONE";
import { RequestModuleParameters } from "@/javascript/OZONE/ajax";
import type { Wikirequest } from "wikirequest";

declare const YAHOO: any;
declare type YahooResponse = any;
declare const fx: any;
declare const WIKIREQUEST: Wikirequest;

// TODO These are externally defined somewhere - where?
const HTTP_SCHEMA = "https";
const URL_HOST = "wikijump.test";

type EditLock = {
  id: unknown;
  secret: unknown;
  revisionId: unknown;
  timeLeft: unknown;
  rangeStart?: unknown;
  rangeEnd?: unknown;
};

export const page = {
  vars: {
    forceLockFlag: false,
    newPage: false,
    editSectionsActive: false,
    editHeadings: [] as unknown[],
    sectionEditButtons: [] as HTMLAnchorElement[],
    locked: false,
    editlock: {} as EditLock,
    ctrle: {} as unknown // Return value of yahoo keylistener
  },

  listeners: {
    editClick: function (_event?: Event): void {
      const pageId = WIKIREQUEST.info.pageId;
      let parms: RequestModuleParameters;
      if (pageId != null) {
        // editing old page
        parms = {
          page_id: pageId,
          mode: 'page',
          wiki_page: WIKIREQUEST.info.requestPageName
        };
      } else {
        // means new page
        Wikijump.page.vars.newPage = true;
        parms = {
          mode: 'page',
          wiki_page: WIKIREQUEST.info.requestPageName
        };
      }
      if (Wikijump.page.vars.forceLockFlag) {
        Wikijump.page.vars.forceLockFlag = false;
        parms.force_lock = 'yes';
      }
      OZONE.ajax.requestModule("Edit/PageEditModule", parms, Wikijump.page.callbacks.editClick);
    },

    append: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId,
        mode: 'append'
      };
      OZONE.ajax.requestModule("Edit/PageEditModule", parms, Wikijump.page.callbacks.editClick);
    },

    editSection: function (_event: Event): void {
      // @ts-expect-error What is `this`?
      const sectionNumber = this.id.replace(/edit-section-b-/, '');
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId,
        mode: 'section',
        section: sectionNumber
      };
      OZONE.ajax.requestModule("Edit/PageEditModule", parms, Wikijump.page.callbacks.editClick);
    },

    historyClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("History/PageHistoryModule", parms, Wikijump.page.callbacks.historyClick);
    },

    filesClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("Files/PageFilesModule", parms, Wikijump.page.callbacks.filesClick);
    },

    blockClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("PageBlock/PageBlockModule", parms, Wikijump.page.callbacks.blockClick);
    },

    moreOptionsClick: function (_event: Event): void {
      // make fx or not?
      if (!document.getElementById("page-options-bottom")) { return; }
      const ofx = new fx.Opacity("page-options-bottom-2", { duration: 200 });
      ofx.setOpacity(0);
      document.getElementById("page-options-bottom-2")!.style.display = "block";
      ofx.custom(0, 1);
      document.getElementById("more-options-button")!.innerHTML = document.getElementById("more-options-button")!.innerHTML.replace(/\+/, '-');
      YAHOO.util.Event.removeListener("more-options-button", "click", Wikijump.page.listeners.moreOptionsClick);
      YAHOO.util.Event.addListener("more-options-button", "click", Wikijump.page.listeners.lessOptionsClick);
      OZONE.visuals.scrollTo('page-options-bottom');
    },

    lessOptionsClick: function (_event: Event): void {
      if (!document.getElementById("page-options-bottom-2")) { return; }
      const ofx = new fx.Opacity("page-options-bottom-2", { duration: 200 });
      ofx.custom(1, 0);
      setTimeout(() => { document.getElementById("page-options-bottom-2")!.style.display = "none"; }, 200);
      document.getElementById("more-options-button")!.innerHTML = document.getElementById("more-options-button")!.innerHTML.replace(/-/, '+');
      YAHOO.util.Event.removeListener("more-options-button", "click", Wikijump.page.listeners.lessOptionsClick);
      YAHOO.util.Event.addListener("more-options-button", "click", Wikijump.page.listeners.moreOptionsClick);
    },

    logoutClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        action: "LoginAction",
        event: "logout"
      };
      OZONE.ajax.requestModule(null, parms, Wikijump.page.callbacks.logoutClick);
    },

    loginClick2: function (_event: Event, _resetRemember: unknown): void {
      /**
       * ?? Possibly newer login callback - similar method is used in upstream
       */
      // start the shader
      const shader = OZONE.dialog.factory.shader();
      shader.show();
      // now create an iframe and position (almost) exactly as the viewport!
      const body = document.getElementsByTagName('body')[0];
      const sIfr = document.createElement('iframe');
      sIfr.id = "login-iframe";
      // TODO: De-Wikijump.com-ize - parameter
      let url = window.location.protocol + '//www.wikijump.com/default--flow/Login__LoginIframeScreen';
      url += '/siteId/' + WIKIREQUEST.info.siteId;
      url += '/categoryId/' + WIKIREQUEST.info.categoryId;
      url += '/themeId/' + WIKIREQUEST.info.themeId;
      url += '/url/' + encodeURIComponent(encodeURIComponent(window.location.href));
      sIfr.src = url;
      sIfr.scrolling = "no";
      sIfr.frameBorder = "0";
      sIfr.style.height = YAHOO.util.Dom.getClientHeight() + "px";
      body.appendChild(sIfr);
    },

    loginClick: function (_event: Event, _resetRemember: unknown): void {
      /**
       * Current login callback
       */
      const url = HTTP_SCHEMA + "://" + URL_HOST + '/auth:login?origUrl=' + encodeURIComponent(window.location.href);
      window.location.href = url;

      // let p = new Object();
      // if(resetRemember){ p.reset = "yes"; }
      // OZONE.ajax.requestModule("Login/LoginModule2", p, Wikijump.page.callbacks.loginClick);
    },

    createAccount: function (_event: Event): void {
      const url = HTTP_SCHEMA + "://" + URL_HOST + '/auth:newaccount?origUrl=' + encodeURIComponent(window.location.href);
      window.location.href = url;

      // OZONE.ajax.requestModule("CreateAccount/CreateAccountStep1Module", null, Wikijump.page.callbacks.createAccount);
    },

    toggleEditSections: function (_event?: Event): void {
      if (!Wikijump.page.vars.editSectionsActive) {
        // check if it is possible to edit sections.

        const pc = document.getElementById("page-content")!;
        const children = pc.children;
        const headings = [];
        for (let i = 0; i < children.length; i++) {
          const tagName = children[i].tagName;
          if (tagName && tagName.toLowerCase().match(/^h[1-6]$/) && children[i].id.match(/^toc/)) {
            headings.push(children[i]);
          }
        }

        if (headings.length === 0) {
          // alert("no isolated sections to edit")
          const w = new OZONE.dialogs.ErrorDialog();
          w.content = "There are no isolated sections to edit.";
          w.show();
          return;
        }

        // count all headings in the page-content
        let allSum = 0;
        const hTypes = ['h1', 'h2', 'h3', 'h4', 'h5', 'h6'];
        for (let i = 0; i < hTypes.length; i++) {
          const theads = pc.getElementsByTagName(hTypes[i]);
          for (let j = 0; j < theads.length; j++) {
            if (theads[j].id.match(/^toc/)) {
              allSum++;
            }
          }
        }
        if (allSum !== headings.length) {
          alert("It seems that headings do not have a valid structure...");
          return;
        }

        const editButtons: HTMLAnchorElement[] = [];
        for (let i = 0; i < headings.length; i++) {
          const edit = document.createElement("a");
          edit.innerHTML = "edit";
          edit.href = "javascript:;";
          edit.className = "edit-section-button";
          edit.id = "edit-section-b-" + headings[i].id.replace(/toc/, '');
          YAHOO.util.Event.addListener(edit, "click", Wikijump.page.listeners.editSection);
          const ef = new fx.Opacity(edit, { duration: 300 });
          ef.setOpacity(0);
          pc.insertBefore(edit, headings[i]);
          ef.custom(0, 1);
          editButtons.push(edit);
        }
        Wikijump.page.vars.editHeadings = headings;
        Wikijump.page.vars.sectionEditButtons = editButtons;
        Wikijump.page.vars.editSectionsActive = true;
      } else {
        const edits = Wikijump.page.vars.sectionEditButtons;
        for (let i = 0; i < edits.length; i++) {
          edits[i].parentNode!.removeChild(edits[i]);
        }
        Wikijump.page.vars.editSectionsActive = false;
      }
    },

    editTags: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        pageId: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("PageTags/PageTagsModule", parms, Wikijump.page.callbacks.editTags);
    },

    siteTools: function (_event: Event): void {
      OZONE.ajax.requestModule("SiteTools/SiteToolsModule", {}, Wikijump.page.callbacks.siteTools);
    },

    backlinksClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        pageId: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("Backlinks/BacklinksModule", parms, Wikijump.page.callbacks.backlinksClick);
    },
    viewSourceClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        pageId: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("ViewSource/ViewSourceModule", parms, Wikijump.page.callbacks.viewSourceClick);
    },

    closeActionArea: function (_event: Event): void {
      const a = document.getElementById("action-area");
      if (a) {
        if (document.getElementById("page-options-bottom")) {
          const myEffect = new fx.ScrollBottom({ duration: 100, transition: fx.sineOut });
          myEffect.scrollTo("page-options-bottom");
        }
        setTimeout(() => {
          document.getElementById("action-area")!.innerHTML = "";
          document.getElementById("action-area")!.style.display = "none";
        }, 200);
      }
    },

    userInfo: function (userId: number): void {
      const parms: RequestModuleParameters = {
        user_id: userId
      };
      OZONE.ajax.requestModule("Users/UserInfoWinModule", parms, Wikijump.page.callbacks.userInfo);
    },

    anonymousUserInfo: function (userString: string): void {
      const parms: RequestModuleParameters = {
        userString: userString
      };
      OZONE.ajax.requestModule("Users/AnonymousInfoWinModule", parms, Wikijump.page.callbacks.userInfo);
    },

    renamePage: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        pageId: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("Rename/RenamePageModule", parms, Wikijump.page.callbacks.renamePage);
    },
    deletePage: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        pageId: WIKIREQUEST.info.pageId,
        delete: "yes"
      };
      OZONE.ajax.requestModule("Rename/RenamePageModule", parms, Wikijump.page.callbacks.renamePage);
    },
    createPageDiscussion: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId,
        action: "ForumAction",
        event: "createPageDiscussionThread"
      };
      OZONE.ajax.requestModule("Empty", parms, Wikijump.page.callbacks.createPageDiscussion);
    },

    flagPageObjectionable: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        path: window.location.pathname
      };
      OZONE.ajax.requestModule('Report/FlagPageModule', parms, Wikijump.page.callbacks.flagPageObjectionable);
    },
    pageBugReport: function (_event: Event): void {
      OZONE.ajax.requestModule('Report/BugReportModule', {}, Wikijump.page.callbacks.pageBugReport);
    },
    pageRate: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        pageId: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule('PageRate/PageRateModule', parms, Wikijump.page.callbacks.pageRate);
    },
    parentClick: function (_event: Event): void {
      const parms: RequestModuleParameters = {
        page_id: WIKIREQUEST.info.pageId
      };
      OZONE.ajax.requestModule("Parent/ParentPageModule", parms, Wikijump.page.callbacks.parentClick);
    },
    passwordRecoveryClick: function (_event: Event): void {
      OZONE.ajax.requestModule("PasswordRecovery/PasswordRecoveryModule", {}, Wikijump.page.callbacks.passwordRecovery);
    },

    foldToc: function (_event: Event): void {
      const eff = new fx.Opacity(document.getElementById("toc-list"), {
        duration: 200,
        onComplete: function () {
          document.getElementById("toc-list")!.style.display = "none";
          const as = document.getElementById("toc-action-bar")!.getElementsByTagName('a');
          as[0].style.display = "none";
          as[1].style.display = '';
        }
      });
      eff.custom(1, 0);
    },
    unfoldToc: function (_event: Event): void {
      const eff = new fx.Opacity(document.getElementById("toc-list"), { duration: 200 });
      eff.setOpacity(0);
      document.getElementById("toc-list")!.style.display = "block";
      eff.custom(0, 1);
      const as = document.getElementById("toc-action-bar")!.getElementsByTagName('a');
      as[1].style.display = "none";
      as[0].style.display = '';
    },

    search: function (_event: Event): void {
      const searchBoxElement = document.getElementById("search-top-box-input") as HTMLInputElement;
      let query = searchBoxElement.value;
      // escape query
      query = encodeURIComponent(query);
      const url = "/search:site/q/" + query;
      window.location.href = url;
      YAHOO.util.Event.preventDefault(_event);
    },

    printClick: function (_event: Event): void {
      // open a new window...
      const url = '/printer--friendly/' + window.location.pathname;
      window.open(url, "_blank", `location=no,menubar=yes,titlebar=no,resizable=yes,scrollbars=yes,width=${screen.width * 0.8},height=${screen.height * 0.8},top=${screen.height * 0.1},left=${screen.width * 0.1}`);
    }
  },

  callbacks: {
    filesClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent('action-area', response.body);
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 200);
    },

    editClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      if (Wikijump.page.vars.newPage) {
        document.getElementById('page-content')!.innerHTML = '';
      }

      if (Wikijump.page.vars.editSectionsActive) {
        Wikijump.page.listeners.toggleEditSections();
      }

      // init
      //@ts-expect-error Shouldn't need to attach to window
      window.editMode = response.mode;

      if (response.locked) {
        // the page has a lock!
        Wikijump.page.vars.locked = true;
        // put output in the window
        OZONE.dialog.factory.shader().show();
        const cont = OZONE.dialog.factory.boxcontainer();
        cont.setContent(response.body);
        cont.showContent();
        return;
      } else {
        Wikijump.page.vars.locked = false;
        const pageId = WIKIREQUEST.info.pageId;
        if (pageId != null) {
          // editing old page
          if (document.getElementById("page-options-bottom")) {
            document.getElementById("page-options-bottom")!.style.display = 'none';
            document.getElementById("page-options-bottom-2")!.style.display = "none";
          }
          if (document.getElementById("page-options-area-bottom")) {
            document.getElementById("page-options-area-bottom")!.style.display = 'none';
          }
        }
        // lock information! (veeeery crucial)
        Wikijump.page.vars.editlock = {
          id: response.lock_id,
          secret: response.lock_secret,
          revisionId: response.page_revision_id,
          timeLeft: response.timeLeft
        };
      }

      // @ts-expect-error Shouldn't be attached to window
      if (window.editMode === 'section') {
        if (response.section == null) {
          alert('Section edit error. Section does not exist');
          return;
        }
        Wikijump.page.vars.editlock.rangeStart = response.rangeStart;
        Wikijump.page.vars.editlock.rangeEnd = response.rangeEnd;

        // insert new div before the heading...
        const headingId = 'toc' + response.section;
        const heading = document.getElementById(headingId)!;
        const aDiv = document.createElement('div');
        aDiv.id = 'edit-section-content';
        const pc = document.getElementById("page-content")!;
        pc.insertBefore(aDiv, heading);
        const re = new RegExp('^h[1-' + heading.tagName.replace(/h/i, '') + ']', 'i');
        let ns = heading.nextElementSibling;
        aDiv.appendChild(heading);
        while (ns != null) {
          if (ns.tagName && ns.tagName.match(re) && ns.id.match(/^toc/)) {
            break;
          }
          const ns0 = ns;
          ns = ns.nextElementSibling;
          aDiv.appendChild(ns0);
        }
        // also move action area below that div.
        if (ns) {
          pc.insertBefore(document.getElementById('action-area')!, ns);
        } else {
          pc.appendChild(document.getElementById('action-area')!);
        }
      }

      OZONE.utils.setInnerHTMLContent('action-area', response.body);
      document.getElementById("action-area")!.style.display = "block";
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 200);
      // @ts-expect-error Return type of KeyListener
      Wikijump.page.vars.ctrle.disable();
    },

    historyClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent('action-area', response.body);
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
    },
    logoutClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      window.location.reload();
    },

    passwordRecovery: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      Wikijump.vars.rsakey = response.key;
      Wikijump.vars.loginSeed = response.seed;
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    },

    createAccount: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    },

    backlinksClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent('action-area', response.body);
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    viewSourceClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      OZONE.utils.setInnerHTMLContent('action-area', response.body);
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },

    userInfo: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }

      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.clickOutsideToClose = true;
      w.show();
    },

    renamePage: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById('action-area')!.innerHTML = response.body;
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    editTags: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById('action-area')!.innerHTML = response.body;
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    blockClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById('action-area')!.innerHTML = response.body;
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    pageRate: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById('action-area')!.innerHTML = response.body.replace(/prw54353/, 'prw54354');
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    siteTools: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById('action-area')!.innerHTML = response.body;
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      OZONE.dialog.hovertip.dominit("site-tools-box", { delay: 700, valign: 'center' });
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    parentClick: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      document.getElementById('action-area')!.innerHTML = response.body;
      document.getElementById("action-area")!.style.display = "block";
      Wikijump.page.utils.addCloseToActionArea();
      setTimeout(() => OZONE.visuals.scrollTo('action-area'), 300);
    },
    createPageDiscussion: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      // create the URI and change page now
      const uri = "/forum/t-" + response.thread_id + '/' + response.thread_unix_title;
      window.location.href = uri;
    },
    flagPageObjectionable: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    },
    pageBugReport: function (response: YahooResponse): void {
      if (!Wikijump.utils.handleError(response)) { return; }
      const w = new OZONE.dialogs.Dialog();
      w.content = response.body;
      w.show();
    }
  },

  utils: {
    addCloseToActionArea: function (): void {
      const cl = document.createElement("a");
      cl.innerHTML = "close";
      cl.href = "javascript:;";
      cl.className = "action-area-close";
      const aa = document.getElementById("action-area")!;
      if (aa.firstChild) {
        aa.insertBefore(cl, aa.firstChild);
      } else {
        aa.appendChild(cl);
      }
      YAHOO.util.Event.addListener(cl, "click", Wikijump.page.listeners.closeActionArea);
    }
  },

  fixers: {
    /**
     * Fix math references to show hover equations.
     */
    fixMathRef: function (): void {
      const erefs = YAHOO.util.Dom.getElementsByClassName("eref");
      let id, eref, equation;
      if (erefs && erefs.length > 0) {
        for (let i = 0; i < erefs.length; i++) {
          eref = erefs[i];
          id = eref.innerHTML;
          equation = document.getElementById("equation-" + id);
          if (equation) {
            // create a tip
            const image = equation.getElementsByTagName('img')[0].cloneNode(true);
            // @ts-expect-error src isn't present on node, but is present on
            // elements. However a simple alternative to cloneNode doesn't
            // exist for elements. TODO
            const text = '<b>Equation (' + id + ')</b><br/><img style="margin: 1em" src="' + image.src + '"/><br/>' +
              '<span style="font-size: 90%">(click to scroll to the equation)</span>';
            OZONE.dialog.hovertip.makeTip(eref, {
              text: text,
              valign: 'center',
              style: { width: 'auto', backgroundColor: 'white' }
            });
          }
        }
      }
    },

    fixFootnoteRef: function (root?: unknown): void {
      const frefs = YAHOO.util.Dom.getElementsByClassName("footnoteref", "a", root);
      for (let i = 0; i < frefs.length; i++) {
        const fref = frefs[i];
        const id = fref.id.replace(/^footnoteref-/, '');
        const footnote = document.getElementById("footnote-" + id)!;

        const content = footnote.innerHTML.replace(/<a.*?<\/a>\. /, '');

        const text = '<b>Footnote ' + id + '.</b><br/>' + '<div style="margin: 0.5em 0">' + content + '</div>' +
          '<span style="font-size: 90%">(click to scroll to footnotes)</span>';
        OZONE.dialog.hovertip.makeTip(fref, {
          text: text,
          valign: 'center',
          smartWidthLimit: 0.7,
          style: { width: 'auto', backgroundColor: 'white' }
        });
      }
    },

    fixBibRef: function (root?: unknown): void {
      const brefs = YAHOO.util.Dom.getElementsByClassName("bibcite", "a", root);
      for (let i = 0; i < brefs.length; i++) {
        const bref = brefs[i];
        const id = bref.id.replace(/bibcite-/, '');
        const bibitem = document.getElementById("bibitem-" + id)!;
        const content = bibitem.innerHTML.replace(/^\s*[0-9]+\.\s*/, '');
        const text = '<b>Reference ' + id + '.</b><br/>' + '<div style="margin: 0.5em 0">' + content + '</div>' + '<span style="font-size: 90%">(click to scroll to bibliography)</span>';
        OZONE.dialog.hovertip.makeTip(bref, {
          text: text,
          valign: 'center',
          smartWidthLimit: 0.7,
          style: { width: 'auto', backgroundColor: 'white' }
        });
      }
    },
    fixDates: function (elementId: string): void {
      OZONE.utils.formatDates(elementId);
    },
    /**
     * Adds listeners to menu li elements
     */
    fixMenu: function (elementId: string): void {
      const r = document.getElementById(elementId);
      if (r == null) { return; }
      const els = r.getElementsByTagName("li");
      for (let i = 0; i < els.length; i++) {
        YAHOO.util.Event.addListener(els[i], "mouseover", function (_event: Event) {
          // @ts-expect-error What is this?
          YAHOO.util.Dom.addClass(this, "sfhover");
        });
        YAHOO.util.Event.addListener(els[i], "mouseout", function (_event: Event) {
          // @ts-expect-error What is this?
          YAHOO.util.Dom.removeClass(this, "sfhover");
        });
      }
    },

    fixEmails: function (element: HTMLElement): void {
      const els = YAHOO.util.Dom.getElementsByClassName("wiki-email", "span", element);
      let el;
      for (let i = 0; i < els.length; i++) {
        el = els[i];
        if (el.innerHTML.match(/^([a-z0-9\-.|_])+#/i)) {
          const s = el.innerHTML.split('#');
          const email = s[0].replace('|', '@');
          let email2 = '';
          for (let j = email.length - 1; j >= 0; j--) {
            email2 += email.charAt(j);
          }
          const text = s[1].replace('|', '@');
          let text2 = '';
          for (let j = text.length - 1; j >= 0; j--) {
            text2 += text.charAt(j);
          }
          const a = document.createElement('a');
          a.href = 'mailto:' + email2;
          a.innerHTML = text2;
          el.innerHTML = '';
          el.appendChild(a);
          el.style.visibility = "visible";
        }
      }
    },

    fixFoldableMenus: function (root: string): void {
      const rootElement = document.getElementById(root);
      if (!rootElement) {
        return;
      }
      const divs = YAHOO.util.Dom.getElementsByClassName("foldable-list-container", 'div', rootElement);
      for (let i = 0; i < divs.length; i++) {
        // get all lists
        const uls = divs[i].getElementsByTagName('ul');
        for (let j = 0; j < uls.length; j++) {
          const ul = uls[j];
          // hide if not a direct descendant of the container
          let parnt = ul.parentNode;
          let direct = true;
          while (parnt && !YAHOO.util.Dom.hasClass(parnt, 'foldable-list-container')) {
            // alert(parnt.tagName+'.'+parnt.className)
            if (parnt.tagName && parnt.tagName === 'LI') {
              direct = false;
              break;
            }
            parnt = parnt.parentNode;
          }
          if (!direct) {
            ul.originalDisplay = ul.style.display;
            ul.style.display = "none";
            YAHOO.util.Dom.addClass(parnt, "folded");
            parnt.eff = new fx.Opacity(ul, { duration: 300 });
            // check if the li has a proper <a> element. if not - add a.
            const tnode = parnt.childNodes[0];
            if (tnode.tagName !== "A") {
              const a = document.createElement('a');
              parnt.insertBefore(a, tnode);
              a.appendChild(tnode);
              a.href = "javascript:;";
            }
          }
        }

        // check if there is any active page here.. if so - unfold the list somehow
        const as = divs[i].getElementsByTagName('a');
        const loc = window.location.pathname;

        for (let j = 0; j < as.length; j++) {
          const href = as[j].href.replace(/^[a-z]*:\/\/[^/]+\/([^/]+).*/, '/$1');
          if (href === loc) {
            let parnt = as[j].parentNode;
            while (
              parnt &&
              !YAHOO.util.Dom.hasClass(parnt, 'foldable-list-container')
            ) {
              if (
                parnt.tagName === 'LI' &&
                YAHOO.util.Dom.hasClass(parnt, "folded")
              ) {
                YAHOO.util.Dom.replaceClass(parnt, "folded", "unfolded");
                const ul = parnt.getElementsByTagName('ul')[0];
                ul.style.display = ul.originalDisplay;
              }
              parnt = parnt.parentNode;
            }
          }
        }
        // attach a listener too
        YAHOO.util.Event.addListener(divs[i], "click", Wikijump.page.fixers._foldableMenuToggle);
      }
    },

    _foldableMenuToggle: function (event: Event): void {
      let li;
      li = YAHOO.util.Event.getTarget(event, true);
      if (
        li.tagName === "A" &&
        li.href !== "#" &&
        li.href !== "javascript:;"
      ) {
        return;
      }
      while (!li.tagName || li.tagName !== 'LI') {
        li = li.parentNode;
      }
      if (!(YAHOO.util.Dom.hasClass(li, "folded") || YAHOO.util.Dom.hasClass(li, "unfolded"))) {
        return;
      }

      if (YAHOO.util.Dom.hasClass(li, "folded")) {
        // unfold
        YAHOO.util.Dom.replaceClass(li, "folded", "unfolded");
        const ul = li.getElementsByTagName('ul')[0];
        li.eff.setOpacity(0);
        ul.style.display = ul.originalDisplay;
        li.eff.custom(0, 1);
      } else {
        // fold
        YAHOO.util.Dom.replaceClass(li, "unfolded", "folded");
        const ul = li.getElementsByTagName('ul')[0];

        ul.style.display = 'none';
      }
    },
    /**
     * Inserts A elements into LI elements if not present.
     */
    fixMenuList: function (rootElementId: string): void {
      const rootElement = document.getElementById(rootElementId);
      if (!rootElement) {
        return;
      }
      const lis = rootElement.getElementsByTagName('li');
      for (let i = 0; i < lis.length; i++) {
        const tnode = lis[i].children[0];
        if (
          tnode.tagName !== "A" &&
          tnode.nodeType === 3 &&
          tnode.innerHTML !== ""
        ) {
          const a = document.createElement('a');
          lis[i].insertBefore(a, tnode);
          a.appendChild(tnode);
          a.href = "javascript:;";
        }
      }
    }

  },

  account: {
    shower: function (_event: Event): void {
      // the listener to show account options
      const ao = document.getElementById("account-options")!;
      if (!('eff' in ao)) {
        // @ts-expect-error Gotta replace this effects library
        ao.eff = new fx.Opacity(ao, { duration: 200 });
      }
      // @ts-expect-error Gotta replace this effects library
      ao.eff.setOpacity(0);
      ao.style.display = "block";
      // @ts-expect-error Gotta replace this effects library
      ao.eff.custom(0, 1);
    },

    closer: function (event: Event): void {
      const ao = document.getElementById("account-options")!;
      const rt = YAHOO.util.Event.getRelatedTarget(event);
      // check if rt is ao or is a child of ao
      let is = false;
      if (rt === ao) is = true;
      if (rt.parentNode === ao) is = true;
      if (rt.parentNode.parentNode === ao) is = true;
      if (rt.parentNode.parentNode.parentNode === ao) is = true;
      if (is === true) return;
      // @ts-expect-error Gotta replace this effects library
      ao.eff.setOpacity(0);
      ao.style.display = "none";
    }
  },

  /* initialize a few things */
  init: function (): void {
    YAHOO.util.Event.addListener("edit-button", "click", Wikijump.page.listeners.editClick);
    YAHOO.util.Event.addListener("pagerate-button", "click", Wikijump.page.listeners.pageRate);
    YAHOO.util.Event.addListener("tags-button", "click", Wikijump.page.listeners.editTags);
    YAHOO.util.Event.addListener("history-button", "click", Wikijump.page.listeners.historyClick);
    YAHOO.util.Event.addListener("files-button", "click", Wikijump.page.listeners.filesClick);
    YAHOO.util.Event.addListener("print-button", "click", Wikijump.page.listeners.printClick);
    YAHOO.util.Event.addListener("site-tools-button", "click", Wikijump.page.listeners.siteTools);
    YAHOO.util.Event.addListener("more-options-button", "click", Wikijump.page.listeners.moreOptionsClick);

    YAHOO.util.Event.addListener("edit-append-button", "click", Wikijump.page.listeners.append);
    YAHOO.util.Event.addListener("edit-sections-button", "click", Wikijump.page.listeners.toggleEditSections);
    YAHOO.util.Event.addListener("backlinks-button", "click", Wikijump.page.listeners.backlinksClick);
    YAHOO.util.Event.addListener("parent-page-button", "click", Wikijump.page.listeners.parentClick);

    YAHOO.util.Event.addListener("view-source-button", "click", Wikijump.page.listeners.viewSourceClick);
    YAHOO.util.Event.addListener("page-block-button", "click", Wikijump.page.listeners.blockClick);
    YAHOO.util.Event.addListener("rename-move-button", "click", Wikijump.page.listeners.renamePage);
    YAHOO.util.Event.addListener("delete-button", "click", Wikijump.page.listeners.deletePage);

    YAHOO.util.Event.addListener("search-top-box-form", "submit", Wikijump.page.listeners.search);

    OZONE.dom.onDomReady(function () {
      OZONE.dialog.hovertip.dominit("html-body", { delay: 700, valign: 'center' });
      Wikijump.page.fixers.fixMenuList("top-bar");
      Wikijump.page.fixers.fixFoldableMenus("side-bar");
      Wikijump.page.fixers.fixMathRef();
      Wikijump.page.fixers.fixFootnoteRef();
      Wikijump.page.fixers.fixBibRef();
      Wikijump.page.fixers.fixDates("html-body");
      Wikijump.page.fixers.fixEmails(document.getElementById("page-content")!);

      const accountButton = document.getElementById("account-topbutton");
      if (accountButton) {
        YAHOO.util.Event.addListener(accountButton, "mousedown", Wikijump.page.account.shower);
        YAHOO.util.Event.addListener("account-options", "mouseout", Wikijump.page.account.closer);
      }
      Wikijump.page.fixers.fixMenu("top-bar");
      Wikijump.page.fixers.fixMenu("side-bar");

      OZONE.visuals.initScroll();
      const not = document.getElementById("notifications-dialog");
      if (not != null) {
        const w = new OZONE.dialogs.Dialog();
        w.content = not.innerHTML;
        w.show();
        setTimeout(() => OZONE.dialog.factory.boxcontainer().centerContent(), 1000);
      }

      // check if to start page editing now
      const path = window.location.pathname;
      if (path.match(/^\/[a-z0-9\-:]+\/edit\/true/)) {
        Wikijump.page.listeners.editClick();
        // force use the template
      }
      // check if need highlighting
      /*
         let path = window.location.pathname;
         if(path.match(/^\/[^\/]+.*?\/highlight\/([^\/]+)(\/|$)/)){
         let htext = path.replace(/^\/[^\/]+.*?\/highlight\/([^\/]+)(\/|$)/, "$1");
         htext = decodeURIComponent(htext);
         OZONE.visuals.highlightText('page-content', htext);
         OZONE.visuals.highlightText('page-title', htext);
         }
       */
    }, "dummy-ondomready-block");

    OZONE.loc.addMessage("close window", "zamknij okno", "pl");
    OZONE.loc.addMessage("close message", "zamknij wiadomość", "pl");
    OZONE.loc.addMessage("Error", "Blad", "pl");
    OZONE.loc.addMessage("Oooops!", "Ups!", "pl");
    OZONE.loc.addMessage("Permission error", "Błąd uprawnień", "pl");

    const ldates = {
      ago: "temu", day: "dzień", days: "dni", hours: "godziny", hour: "godzina", minutes: "minuty", minute: "minuta", seconds: 'sekundy', second: 'sekunda'
    };

    OZONE.loc.addMessages(ldates, "pl");

    // attache ctrl+e for editing
    // handle ctrl+s
    const ctrle = new YAHOO.util.KeyListener(
      document, { keys: 69, ctrl: true }, function (_type: unknown, e: unknown) {
        // @ts-expect-error I do not know what e is
        e = e[1];
        Wikijump.page.listeners.editClick();
        YAHOO.util.Event.stopEvent(e);
      }
    );
    ctrle.enable();
    Wikijump.page.vars.ctrle = ctrle;
  }
};
