

Wikijump.modules.PageEditModule = {};

Wikijump.modules.PageEditModule.vars = {
	stopCounterFlag: false,
	inputFlag: false, // changed to true by any input
	lastInput: (new Date()).getTime(), // last input.
	savedSource: '' // source saved on server
};

Wikijump.modules.PageEditModule.listeners = {
	cancel: function(e){

		var r = YAHOO.util.Event.removeListener(window, "beforeunload", Wikijump.modules.PageEditModule.listeners.leaveConfirm);
		YAHOO.util.Event.removeListener(window, "unload", Wikijump.modules.PageEditModule.listeners.leavePage);

		if($('wikijump-disable-locks-flag')){
			window.location.href='/'+WIKIREQUEST.info.requestPageName;
			return;
		}
		var parms = new Object();
		parms['lock_id'] = Wikijump.page.vars.editlock.id;
		parms['lock_secret'] = Wikijump.page.vars.editlock.secret;
		parms['action']="WikiPageAction";
		parms['event']="removePageEditLock";
		OZONE.ajax.requestModule("Empty",parms, Wikijump.modules.PageEditModule.callbacks.cancel);

	},

	preview: function(e){
		var params = OZONE.utils.formToArray("edit-page-form");
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;
		params['page_unix_name'] = WIKIREQUEST.info.requestPageName;
		if(WIKIREQUEST.info.pageId){
			params['pageId'] = WIKIREQUEST.info.pageId;
		}

		// TODO we are going to replace this entire damn thing with Sheaf
		alert('Page previews are disabled!');
	},

	save: function(e){
		var t2 = new OZONE.dialogs.WaitBox(); //global??? pheeee...
		t2.content="Saving page..."	;
		t2.show();

		var params = OZONE.utils.formToArray("edit-page-form");
		params['action'] = 'WikiPageAction';
		params['event'] = 'savePage';
		params['wiki_page'] = WIKIREQUEST.info.requestPageName;
		params['lock_id'] = Wikijump.page.vars.editlock.id;
		if( WIKIREQUEST.info.pageId) {params['page_id'] = WIKIREQUEST.info.pageId;}
		params['lock_secret'] = Wikijump.page.vars.editlock.secret;
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;

		/* Handle tags. */
		try{
			path = location.href.toString();
			if(zz = path.match(/\/tags\/([^\/]+)/)){
				// set the title
				params.tags = decodeURIComponent(zz[1]);
			}
		}catch(e){}
		OZONE.ajax.requestModule("Empty",params,Wikijump.modules.PageEditModule.callbacks.save);

	},
	saveAndContinue: function(e){
		var t2 = new OZONE.dialogs.WaitBox(); //global??? pheeee...
		t2.content="Saving page..."	;
		t2.show();

		var params = OZONE.utils.formToArray("edit-page-form");
		params['action'] = 'WikiPageAction';
		params['event'] = 'savePage';
		params['wiki_page'] = WIKIREQUEST.info.requestPageName;
		params['lock_id'] = Wikijump.page.vars.editlock.id;
		if( WIKIREQUEST.info.pageId) {params['page_id'] = WIKIREQUEST.info.pageId;}
		params['lock_secret'] = Wikijump.page.vars.editlock.secret;
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;
		params['and_continue'] = "yes";

		OZONE.ajax.requestModule("Empty",params,Wikijump.modules.PageEditModule.callbacks.saveAndContinue);

	},

	changeInput: function(e){
		Wikijump.modules.PageEditModule.vars.inputFlag = true;

		Wikijump.modules.PageEditModule.vars.lastInput = (new Date()).getTime();
		Wikijump.modules.PageEditModule.utils.timerSetTimeLeft(15*60);
	},

	leaveConfirm: function(e){
		if(Wikijump.modules.PageEditModule.utils.sourceChanged()){
			e.returnValue='If you leave this page, all the unsaved changes will be lost.';
		}
	},

	/**
	 * When a page is left we realy _should_ release the lock.
	 */
	leavePage: function(e){
		// release lock
		var parms = new Object();
		parms['action']="WikiPageAction";
		parms['event']="removePageEditLock";
		parms['lock_id'] = Wikijump.page.vars.editlock.id;
		parms['lock_secret'] = Wikijump.page.vars.editlock.secret;
		OZONE.ajax.requestModule("Empty",parms, Wikijump.modules.PageEditModule.callbacks.forcePageEditLockRemove);

		/* DO WE NEED THIS INFO????
		if(Wikijump.modules.PageEditModule.utils.sourceChanged()){
			alert("You have closed or left the window while editing a page.\n" +
				"If you have made any changes without saving them they are lost now.\n" +
				"The page edit lock has been removed.");
		}*/

	},

	leavePageRemoveLock: function(e){
		alert("deprectated!!!");

	},

	forcePageEditLockRemove: function(e){
		Wikijump.page.vars.forceLockFlag = true;
		OZONE.dialog.cleanAll();
		Wikijump.page.listeners.editClick(null);
	},

	forceLockIntercept: function(e){
		var params = new Object();
		params['action'] = 'WikiPageAction';
		params['event'] = 'forceLockIntercept';
		params['wiki_page'] = WIKIREQUEST.info.requestPageName;
		if(WIKIREQUEST.info.pageId) {params['page_id'] = WIKIREQUEST.info.pageId;}
		params['lock_id'] = Wikijump.page.vars.editlock.id;
		params['lock_secret'] = Wikijump.page.vars.editlock.secret;
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;

		OZONE.ajax.requestModule("Empty",params,Wikijump.modules.PageEditModule.callbacks.forceLockIntercept);
	},
	recreateExpiredLock: function(e){
		var params = new Object();
		params['action'] = 'WikiPageAction';
		params['event'] = 'recreateExpiredLock';
		params['wiki_page'] = WIKIREQUEST.info.requestPageName;
		params['lock_id'] = Wikijump.page.vars.editlock.id;
		if(WIKIREQUEST.info.pageId) {params['page_id'] = WIKIREQUEST.info.pageId;}
		params['lock_secret'] = Wikijump.page.vars.editlock.secret;
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;
		params['since_last_input'] = 0; // seconds since last input

		OZONE.ajax.requestModule("Empty",params,Wikijump.modules.PageEditModule.callbacks.recreateExpiredLock);
	},

	templateChange: function(e){
		if(! $("page-templates")){ return;}
		var templateId = $("page-templates").value;
		var change = true;
		if(Wikijump.modules.PageEditModule.utils.sourceChanged()){
			change = confirm("It seems you have already changed the page.\n" +
					"Changing the initial template now will reset the edited page.\n" +
					"Do you want to change the initial content?");
		}
		if(change){
			Wikijump.modules.PageEditModule.vars.templateId = templateId;
			if(templateId == null || templateId == ""){
				$("edit-page-textarea").value = '';
			} else {
				var p = new Object();
				p['page_id'] = templateId;
				OZONE.ajax.requestModule("Edit/TemplateSourceModule", p,Wikijump.modules.PageEditModule.callbacks.templateChange );
			}

		}else{
			$("page-templates").value = Wikijump.modules.PageEditModule.vars.templateId;
		}
	},
	viewDiff: function(e){
		var params = OZONE.utils.formToArray("edit-page-form");
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;
		OZONE.ajax.requestModule("Edit/PageEditDiffModule",params,Wikijump.modules.PageEditModule.callbacks.viewDiff);

	},
	confirmExpiration: function(e){
		Wikijump.modules.PageEditModule.utils.deactivateAll();
		OZONE.dialog.cleanAll();
	},

	closeDiffView: function(e){
		OZONE.visuals.scrollTo('action-area');
		setTimeout('$("view-diff-div").innerHTML=""', 250);
	}
}

Wikijump.modules.PageEditModule.callbacks = {
	preview: function(response){
		if(!Wikijump.utils.handleError(response)) {return;}

		var message = document.getElementById("preview-message").innerHTML;
		OZONE.utils.setInnerHTMLContent("action-area-top", message);

		var title = response.title;
		OZONE.utils.setInnerHTMLContent("page-title", title);
		OZONE.visuals.scrollTo("container");
		$("page-content").innerHTML = response.body;
		Wikijump.page.fixers.fixEmails($("page-content"));

		Wikijump.modules.PageEditModule.utils.stripAnchors("page-content", "action-area");
	},

	viewDiff: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("view-diff-div").innerHTML = r.body;
		OZONE.visuals.scrollTo("view-diff-div");
	},

	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		// check for errors?
		if(r.noLockError){
			// non recoverable. not saved.
			Wikijump.modules.PageEditModule.utils.timerStop();

			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.body;
			w.show();

			if(r.nonrecoverable == true){
				Wikijump.modules.PageEditModule.utils.deactivateAll();
			}

			return;
		}
		Wikijump.modules.PageEditModule.utils.timerStop();

		setTimeout('OZONE.dialog.factory.boxcontainer().hide({smooth: true})',400);
		setTimeout('var t2 = new OZONE.dialogs.SuccessBox(); t2.timeout=10000; t2.content="Page saved!";t2.show()', 600);
		var newUnixName = WIKIREQUEST.info.requestPageName;
		if(r.pageUnixName){
			newUnixName = r.pageUnixName;
		}
		setTimeout('window.location.href="/'+newUnixName+'"',1500);

		YAHOO.util.Event.removeListener(window, "beforeunload", Wikijump.modules.PageEditModule.listeners.leaveConfirm);
		YAHOO.util.Event.removeListener(window, "unload", Wikijump.modules.PageEditModule.listeners.leavePage);
	},
	saveAndContinue: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		if(r.noLockError){
			Wikijump.modules.PageEditModule.utils.timerStop();
			var cont = OZONE.dialog.factory.boxcontainer();
			cont.setContent(r.body);
			cont.showContent();
			if(r.nonrecoverable == true){
				Wikijump.modules.PageEditModule.utils.deactivateAll();
			}
			return;
		}
		setTimeout('OZONE.dialog.factory.boxcontainer().hide({smooth: true})',400);
		setTimeout('var t2 = new OZONE.dialogs.SuccessBox(); t2.content="Page saved!";t2.show()', 600);
		setTimeout('OZONE.dialog.cleanAll()',2000);
		Wikijump.modules.PageEditModule.utils.updateSavedSource();
		Wikijump.page.vars.editlock.revisionId = r.revisionId;
	},

	cancel: function(response){
		if(!Wikijump.utils.handleError(response)) {return;}
		window.location.href='/'+WIKIREQUEST.info.requestPageName;
	},

	forcePageEditLockRemove: function(response){
		if(!Wikijump.utils.handleError(response)) {return;}
		Wikijump.page.listeners.editClick(null);
	},

	forceLockIntercept: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		if(r.error){
			alert('Unexpected error');
			return;
		}
		if(r.nonrecoverable == true){
			var cont = OZONE.dialog.factory.boxcontainer();
			cont.setContent(r.body);
			cont.showContent();
		}
		Wikijump.modules.PageEditModule.utils.timerSetTimeLeft(r.timeLeft);
		Wikijump.modules.PageEditModule.utils.timerStart();
		Wikijump.page.vars.editlock.id = r['lock_id'];
		Wikijump.page.vars.editlock.secret = r['lock_secret'];
		var t2 = new OZONE.dialogs.SuccessBox(); //global??? pheeee...
		t2.content="Lock successfully acquired";
		t2.show();
	},

	updateLock: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		// check for errors?

		if(r.noLockError){
			OZONE.dialog.factory.shader().show();
			var cont = OZONE.dialog.factory.boxcontainer();
			cont.setContent(r.body);
			cont.showContent();

			if(r.nonrecoverable == true){
				Wikijump.modules.PageEditModule.utils.deactivateAll();
			}
			Wikijump.modules.PageEditModule.utils.timerStop();
			YAHOO.util.Event.removeListener(window, "beforeunload", Wikijump.modules.PageEditModule.listeners.leaveConfirm);
			YAHOO.util.Event.removeListener(window, "unload", Wikijump.modules.PageEditModule.listeners.leavePage);
			return;
		}

		if(r.lockRecreated){
			Wikijump.page.vars.editlock.id = r.lockId;
			Wikijump.page.vars.editlock.secret = r.lockSecret;

		}
		Wikijump.modules.PageEditModule.utils.timerSetTimeLeft(r.timeLeft);

	},

	lockExpired: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		OZONE.dialog.factory.shader().show();
		var cont = OZONE.dialog.factory.boxcontainer();
		cont.setContent(r.body);
		cont.showContent();
	},
	recreateExpiredLock: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		if(!r.lockRecreated){
			OZONE.dialog.factory.shader().show();
			var cont = OZONE.dialog.factory.boxcontainer();
			cont.setContent(r.body);
			cont.showContent();
		} else {
			Wikijump.page.vars.editlock.id = r.lockId;
			Wikijump.page.vars.editlock.secret = r.lockSecret;
			Wikijump.modules.PageEditModule.utils.timerSetTimeLeft(r.timeLeft);
			Wikijump.modules.PageEditModule.utils.timerStart();
			Wikijump.modules.PageEditModule.vars.lastInput = (new Date()).getTime();
			var t2 = new OZONE.dialogs.SuccessBox(); //global??? pheeee...
			t2.content="Lock succesfully acquired.";
			t2.show();
		}
	},
	templateChange: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		if(r.body!=null && r.body != ""){
			$("edit-page-textarea").value = r.body;
		}

	}

}

Wikijump.modules.PageEditModule.utils = {

	sourceChanged: function() {
		var a = OZONE.utils.formToArray("edit-page-form");
		return (Wikijump.modules.PageEditModule.vars.savedSource != a["source"]);
	},

	updateSavedSource: function() {
		var a = OZONE.utils.formToArray("edit-page-form");
		Wikijump.modules.PageEditModule.vars.savedSource = a["source"];
	},

	stripAnchors: function(elementId, excludeElement){
		var el =  $(elementId);
		if(excludeElement){
			excludeElement = $(excludeElement);
		}
		if(el){
			var anchors = el.getElementsByTagName("a");

			for(i=0; i<anchors.length; i++){
				if(excludeElement == null || !YAHOO.util.Dom.isAncestor(excludeElement, anchors[i])){
					var href = anchors[i].href;
					anchors[i].href = "javascript:;";
					anchors[i].onclick = null;
					anchors[i].target="_self";
					YAHOO.util.Event.purgeElement(anchors[i]);
					YAHOO.util.Event.addListener(anchors[i], "click", Wikijump.modules.PageEditModule.utils.leavePageWarning);
				}
			}
		}

	},
	/**
	 * Replaces anchors with dumb anchors when editing
	 */
	stripAnchorsAll: function(){
		Wikijump.modules.PageEditModule.utils.stripAnchors("html-body", "action-area");

	},

	leavePageWarning: function(){
		alert("Oooops... You should not leave the page while editing it.\nTo abort editing please use the \"cancel\" button below the edit area.");
	},

	updateActiveButtons: function(){
		var el = $("edit-save-continue-button");
		if(el) {
			el.disabled  = false;
			YAHOO.util.Dom.removeClass(el, "disabled");
		}
		$("edit-save-button").disabled = false;
		YAHOO.util.Dom.removeClass($("edit-save-button"), "disabled");
	},

	deactivateAll: function(){
		// deactivates all the buttons, e.g. when lock is intercepted
		var el;
		el = $("edit-save-continue-button")
		if(el) el.disabled = true;
		$("edit-save-button").disabled = true;
		$("lock-info").style.display = "none";
		YAHOO.util.Event.removeListener("edit-page-form", "keypress", Wikijump.modules.PageEditModule.listeners.changeInput);
		YAHOO.util.Event.removeListener("edit-page-textarea", "change",Wikijump.modules.PageEditModule.listeners.changeInput);
		YAHOO.util.Event.removeListener(window, "beforeunload", Wikijump.modules.PageEditModule.listeners.leaveConfirm);
		YAHOO.util.Event.removeListener(window, "unload", Wikijump.modules.PageEditModule.listeners.leavePage);
		Wikijump.modules.PageEditModule.vars.stopCounterFlag = true;
	},

	startLockCounter: function(){
		Wikijump.modules.PageEditModule.vars.counterStart = (new Date()).getTime();
		Wikijump.modules.PageEditModule.vars.lockLastUpdated = (new Date()).getTime();
		Wikijump.modules.PageEditModule.utils.updateLockCounter();
		Wikijump.modules.PageEditModule.vars.counterEmergency = false;

	},
	updateLockCounter: function(){
		var sec =  (new Date()).getTime() - Wikijump.modules.PageEditModule.vars.counterStart;
		sec = Math.round(15*60 - sec*0.001);
		OZONE.utils.setInnerHTMLContent("lock-timer", sec);
		if(sec < 120 && Wikijump.modules.PageEditModule.vars.counterEmergency == false){
			$("lock-timer").style.color="red";
			$("lock-timer").style.textDecoration = "blink";
			Wikijump.modules.PageEditModule.vars.counterEmergency = true;
		}
		setTimeout("Wikijump.modules.PageEditModule.utils.updateLockCounter()", 1000);
	},

	timerSetTimeLeft: function(timeLeft){
		Wikijump.modules.PageEditModule.vars.lockExpire = (new Date()).getTime() + timeLeft*1000; // in miliseconds.
	},

	timerTick: function(){
		var secLeft = Wikijump.modules.PageEditModule.vars.lockExpire - (new Date()).getTime();
		secLeft = Math.round(secLeft*0.001);
		$("lock-timer").innerHTML = secLeft;

		if(secLeft <=0 ){
			Wikijump.modules.PageEditModule.utils.lockExpired();
			return;
		}

		var sinceLastUpdate = (new Date()).getTime() - Wikijump.modules.PageEditModule.vars.lockLastUpdated;
		if(sinceLastUpdate*0.001 >= 60 || (secLeft<60 && Wikijump.modules.PageEditModule.vars.inputFlag)){
			Wikijump.modules.PageEditModule.vars.inputFlag = false;
			Wikijump.modules.PageEditModule.vars.lockLastUpdated = (new Date()).getTime();
			Wikijump.modules.PageEditModule.utils.updateLock();

		}

		// do some action if conditions....

	},

	timerStart: function(){
		if($('wikijump-disable-locks-flag')){return;}
		Wikijump.modules.PageEditModule.vars.timerId = setInterval('Wikijump.modules.PageEditModule.utils.timerTick()', 1000);
	},
	timerStop: function(){
		if($('wikijump-disable-locks-flag')){return;}
		clearInterval(Wikijump.modules.PageEditModule.vars.timerId);
	},
	/**
	 * Send a request to a server to update lock.
	 */
	updateLock: function(){
		if($('wikijump-disable-locks-flag')){return;}
		var secSinceLastInput = Math.round(((new Date()).getTime() - Wikijump.modules.PageEditModule.vars.lastInput)*0.001);
		var params = new Object();
		params['action'] = 'WikiPageAction';
		params['event'] = 'updateLock';
		params['wiki_page'] = WIKIREQUEST.info.requestPageName;
		params['lock_id'] = Wikijump.page.vars.editlock.id;
		if(WIKIREQUEST.info.pageId){	params['page_id'] = WIKIREQUEST.info.pageId;}
		params['lock_secret'] = Wikijump.page.vars.editlock.secret;
		params['revision_id'] = Wikijump.page.vars.editlock.revisionId;
		params['since_last_input'] = secSinceLastInput; //0; // seconds since last input

		OZONE.ajax.requestModule("Empty",params,Wikijump.modules.PageEditModule.callbacks.updateLock);
	},

	lockExpired: function(){
		Wikijump.modules.PageEditModule.utils.timerStop();

		OZONE.ajax.requestModule("Edit/LockExpiredWinModule", null, Wikijump.modules.PageEditModule.callbacks.lockExpired);
	}
}

Wikijump.modules.PageEditModule.init = function(){
	if(Wikijump.page.vars.locked == true){
		OZONE.utils.formatDates();
	} else {
		/* attach listeners */

		YAHOO.util.Event.addListener("update-lock", "click", Wikijump.modules.PageEditModule.utils.updateLock);
		YAHOO.util.Event.addListener("edit-page-form", "keypress", Wikijump.modules.PageEditModule.listeners.changeInput);
		YAHOO.util.Event.addListener("edit-page-textarea", "keydown", Wikijump.modules.PageEditModule.listeners.changeInput);
		YAHOO.util.Event.addListener(window, "beforeunload", Wikijump.modules.PageEditModule.listeners.leaveConfirm);
		YAHOO.util.Event.addListener(window, "unload", Wikijump.modules.PageEditModule.listeners.leavePage);

		Wikijump.modules.PageEditModule.utils.stripAnchorsAll();
		Wikijump.modules.PageEditModule.utils.updateSavedSource();
		Wikijump.modules.PageEditModule.utils.updateActiveButtons();
		var path = window.location.pathname;
		var zz;
		if(zz = path.match(/^\/[a-z0-9\-:]+\/edit\/true\/t\/([0-9]+)/)){
			// force use the template

			var templateId = zz[1]	;
			$("page-templates").value = templateId;
		}

		try{
			if(zz = path.match(/\/title\/([^\/]+)/)){
				// set the title
				$("edit-page-title").value = decodeURIComponent(zz[1]);
			}
		}catch(e){}

		if( !WIKIREQUEST.info.pageId){
			// new page - init templates!
			Wikijump.modules.PageEditModule.listeners.templateChange(null);
		}

		Wikijump.modules.PageEditModule.utils.timerSetTimeLeft(60*15);
		 Wikijump.modules.PageEditModule.vars.lockLastUpdated = (new Date().getTime());
		Wikijump.modules.PageEditModule.utils.timerStart();

        var form = $j("#edit-page-form").hasClass('edit-with-form');
        if (! form) {
            Wikijump.Editor.init("edit-page-textarea", "wd-editor-toolbar-panel");
        } else {

            // jquery block for forms
            (function() {
                var $ = $j;

                var update_pagepath_value = function(select) {
                    var el = select;
                    while (! el.hasClass("field-pagepath-chooser")) {
                        el = el.parent();
                    }
                    var link = new Array();
                    var append = true;
                    var new_page_title = $('.new_page_title', el);
                    var new_page_parent = $('.new_page_parent', el);
                    var category = $('.category', el).val();

                    new_page_title.val('');
                    new_page_parent.val('');

                    $("select", el).each(function() {
                        val = $(this).val();
                        if (val == '') {
                            append = false;
                        } else if (append) {
                            if (val == '+') {
                                append = false;
                                var title = $('input.text[type=text]', el).val();
                                if (title) {
                                    title = title.replace(']]', ' ');
                                    new_page_title.val(title);
                                    link.push('[[[' + category + ':' + title + ' | ' + title + ']]]');
                                }
                            } else {
                                var title = $('option[selected]', this).text().replace(']]', ' ');
                                new_page_parent.val(val);
                                link.push('[[[' + val + ' | ' + title + ']]]');
                            }
                        }
                    });
                    $('.value', el).val(link.join(' / '));
                };

                var onchange = function() {
                    var select = $(this);
                    update_pagepath_value(select);

                    var span = select.parent().children("span");
                    var selected = select.val();
                    if (selected == '+') {
                        span.text('');
                        var p = select.parent();
                        p.append($('<input type="text" class="text" value="Enter new page name"/>').change(function() {
                            update_pagepath_value($(this));
                        }));
                        p.append(' ');
                        p.append($('<a href="javascript:;">[x]</a>').click(function() {
                            select.show();
                            $('option[selected]', select).removeAttr('selected');
                            $('input', p).remove();
                            $(this).remove();
                            update_pagepath_value(select);
                        }));
                        select.hide();
                        $('input', p).focus().select();
                    } else if (selected == '') {
                        span.text('');
                    } else {
                        span.text(" / Loading...");
                        var q = selected.split(":")[0] + ':';
                        $.getJSON('/quickmodule.php', {s: WIKIREQUEST.info.siteId, q: q, module: 'PageLookupQModule', 'parent': selected}, function(data, textStatus) {
                            var pages = data.pages;
                            var s = $('<select/>')
                            s.append('<option value=""/>');
                            if (pages) {
                                for (var i = 0; i < pages.length; i++) {
                                    var o = $('<option/>');
                                    o.text(pages[i].title);
                                    o.attr('value', pages[i].unix_name);
                                    s.append(o);
                                }
                            }
                            s.append('<option value="+" style="border-top: 1px #666 solid; font-weight: bold">Create new</option>');
                            span.text(' / ');
                            s.change(onchange);
                            span.append(s);
                            span.append('<span/>');
                        });
                    }
                };
                $j("#edit-page-form .field-pagepath-chooser select").change(onchange);
            })();
        }

		var limiter = new OZONE.forms.lengthLimiter("edit-page-comments", "comments-charleft", 200);
		OZONE.dialog.cleanAll();

		// clear all visible hovers
		OZONE.dialog.hovertip.hideAll();

		// prevent backspace from going back
		YAHOO.util.Event.addListener(window, 'keypress', function(e){
			var kc = YAHOO.util.Event.getCharCode(e);
			if(kc == 8){
				var t = YAHOO.util.Event.getTarget(e, true);
				if(t.tagName.toLowerCase() != 'input' && t.tagName.toLowerCase() != 'textarea'){
					YAHOO.util.Event.stopEvent(e);
				}
			}

		});

		// handle ctrl+s
		var ctrls = new YAHOO.util.KeyListener(document, {keys:83, ctrl:true}, function(type,e){
				e = e[1];
				Wikijump.modules.PageEditModule.listeners.save(e);
				YAHOO.util.Event.stopEvent(e);
			}
		);
		ctrls.enable();
        if (form) {
            // focus first field?
        } else {
    		$("edit-page-textarea").focus();
        }

	}
}

setTimeout("Wikijump.modules.PageEditModule.init()", 10);
