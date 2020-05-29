/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.page.vars = {};
WIKIDOT.page.vars.forceLockFlag = false;

WIKIDOT.page.listeners = {
	editClick: function(e){
		var pageId = WIKIREQUEST.info.pageId;
		if(pageId!=null){
			// editing old page
			var parms = new Object();
			parms['page_id']=pageId;
			parms['mode']='page';
			parms['wiki_page'] = WIKIREQUEST.info.requestPageName;
		} else {
			// means new page
			WIKIDOT.page.vars.newPage = true;
			var parms = new Object();
			parms['mode'] = 'page';
			parms['wiki_page'] = WIKIREQUEST.info.requestPageName;
		}
		if(WIKIDOT.page.vars.forceLockFlag == true){
			WIKIDOT.page.vars.forceLockFlag = false;
			parms['force_lock'] = 'yes';
		}
		OZONE.ajax.requestModule("edit/PageEditModule",parms,WIKIDOT.page.callbacks.editClick);
	},
	
	append: function(e){
		var parms = new Object();
		parms['page_id']=WIKIREQUEST.info.pageId;
		parms['mode']='append';
		OZONE.ajax.requestModule("edit/PageEditModule",parms,WIKIDOT.page.callbacks.editClick);
	},
	
	editSection: function(e){
		var sectionNumber = this.id.replace(/edit\-section\-b\-/,'');
		var parms = new Object();
		parms['page_id']=WIKIREQUEST.info.pageId;
		parms['mode']='section';
		parms['section'] = sectionNumber;
		OZONE.ajax.requestModule("edit/PageEditModule",parms,WIKIDOT.page.callbacks.editClick);
	},
		
	historyClick: function(e){
		var parms = new Object();
		parms['page_id']=WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("history/PageHistoryModule",parms,WIKIDOT.page.callbacks.historyClick);
	},
	
	filesClick: function(e){
		var parms = new Object();
		parms['page_id']=WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("files/PageFilesModule",parms,WIKIDOT.page.callbacks.filesClick);
		
	},
	blockClick: function(e){
		var parms = new Object();
		parms['page_id']=WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("pageblock/PageBlockModule",parms,WIKIDOT.page.callbacks.blockClick);
		
	},
	
	moreOptionsClick: function(e){
		// make fx or not? ;-)
		if(!$("page-options-bottom")){return;}
		var ofx = new fx.Opacity("page-options-bottom-2",{duration:200});
		ofx.setOpacity(0);
		$("page-options-bottom-2").style.display="block";
		ofx.custom(0,1);
		$("more-options-button").innerHTML = $("more-options-button").innerHTML.replace(/\+/, '-');
		YAHOO.util.Event.removeListener("more-options-button", "click", WIKIDOT.page.listeners.moreOptionsClick);
		YAHOO.util.Event.addListener("more-options-button", "click", WIKIDOT.page.listeners.lessOptionsClick);
		OZONE.visuals.scrollTo('page-options-bottom');
	},
	
	lessOptionsClick: function(e){
		if(!$("page-options-bottom-2")){return;}
		var ofx = new fx.Opacity("page-options-bottom-2",{duration:200});
		ofx.custom(1,0);
		setTimeout('document.getElementById("page-options-bottom-2").style.display="none"', 200);
		$("more-options-button").innerHTML = $("more-options-button").innerHTML.replace(/\-/, '+');;
		YAHOO.util.Event.removeListener("more-options-button", "click", WIKIDOT.page.listeners.lessOptionsClick);
		YAHOO.util.Event.addListener("more-options-button", "click", WIKIDOT.page.listeners.moreOptionsClick);
	},
	
	logoutClick: function(e){
		var p = new Object();
		p.action = "LoginAction";
		p.event = "logout";
		OZONE.ajax.requestModule(null,p,WIKIDOT.page.callbacks.logoutClick);
	},
	
	loginClick2: function(e, resetRemember){
		// start the shader
		var shader = OZONE.dialog.factory.shader();
		shader.show();
		// now create an iframe and position (almost) exactly as the viewport!
		var body = document.getElementsByTagName('body').item(0);
		var sIfr = document.createElement('iframe');
		sIfr.id="login-iframe";
		// TODO: De-Wikidot.com-ize - parameter
		var url=window.location.protocol+'//www.wikidot.com/default--flow/login__LoginIframeScreen';
		url += '/siteId/'+WIKIREQUEST.info.siteId;
		url += '/categoryId/'+WIKIREQUEST.info.categoryId;
		url += '/themeId/'+WIKIREQUEST.info.themeId;
		url += '/url/'+ encodeURIComponent(encodeURIComponent(window.location.href));
		
		sIfr.src=url;
		sIfr.scrolling="no";
		
		sIfr.frameBorder=0;
		sIfr.style.height = YAHOO.util.Dom.getClientHeight()+"px";
		
		
		
		
		body.appendChild(sIfr);
		
	},
	
	loginClick: function(e, resetRemember){
		var url = 'http://'+URL_HOST+'/auth:login?origUrl=' + encodeURIComponent(window.location.href);
		window.location.href = url;
		return;
		//var p = new Object();
		//if(resetRemember){ p.reset = "yes"; }
		//OZONE.ajax.requestModule("login/LoginModule2", p, WIKIDOT.page.callbacks.loginClick);
		
	},
	
	loginClick0: function(e, resetRemember){
		var p = new Object();
		if(resetRemember){ p.reset = "yes"; }
		OZONE.ajax.requestModule("login/LoginModule", p, WIKIDOT.page.callbacks.loginClick);
	},
	
	createAccount: function(e){
		var url = 'http://'+URL_HOST+'/auth:newaccount?origUrl=' + encodeURIComponent(window.location.href);
		window.location.href = url;
		return;
		//OZONE.ajax.requestModule("createaccount/CreateAccountModule", null, WIKIDOT.page.callbacks.createAccount);
	},
	
	toggleEditSections: function(e){
		if(WIKIDOT.page.vars.editSectionsActive == false){
			
			// check if it is possible to edit sections.
			
			var pc = $("page-content");
			var children = pc.childNodes;
			var headings = new Array();
			for(var i=0; i<children.length; i++){
				var tagName = children[i].tagName;
				if(tagName && tagName.toLowerCase().match(/^h[1-6]$/) && children[i].id.match(/^toc/)){
					headings.push(children[i]);
				}
			}
			
			if(headings.length == 0){
				//alert("no isolated sections to edit")
				var w = new OZONE.dialogs.ErrorDialog();
				w.content = "There are no isolated sections to edit.";
				w.show();
				return;
			}
			
			// count all headings in the page-content
			var allSum = 0;
			var hTypes = ['h1', 'h2', 'h3', 'h4', 'h5', 'h6'];
			for(var i=0; i<hTypes.length; i++){
				var theads = pc.getElementsByTagName(hTypes[i]);
				for(var j=0; j<theads.length; j++){
					if(theads[j].id.match(/^toc/)){
						allSum++;
					}
				}
			}
			if(allSum != headings.length){
				alert("It seems that headings do not have a valid structure...");
				return;
			}
			
			var editButtons = new Array();
			for(var i=0; i<headings.length; i++){
				var edit = document.createElement("a");
				edit.innerHTML="edit";
				edit.href="javascript:;"; 
				edit.className = "edit-section-button";
				edit.id="edit-section-b-"+headings[i].id.replace(/toc/,'');
				YAHOO.util.Event.addListener(edit, "click", WIKIDOT.page.listeners.editSection );
				var ef = new fx.Opacity(edit, {duration:300});
				ef.setOpacity(0);
				pc.insertBefore(edit,headings[i]);
				ef.custom(0,1);
				editButtons.push(edit);
			}
			WIKIDOT.page.vars.editHeadings = headings;
			WIKIDOT.page.vars.sectionEditButtons = editButtons;
			WIKIDOT.page.vars.editSectionsActive = true;
		} else {
			var edits = WIKIDOT.page.vars.sectionEditButtons;
			for(var i=0; i<edits.length; i++){
				edits[i].parentNode.removeChild(edits[i]);
			}
			WIKIDOT.page.vars.editSectionsActive = false;
			return;
		}
	},
	
	editTags: function(e){
		var p = new Object();
		p['pageId'] =  WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("pagetags/PageTagsModule", p, WIKIDOT.page.callbacks.editTags);
		
	},
	
	siteTools: function(e){
		OZONE.ajax.requestModule("sitetools/SiteToolsModule", null, WIKIDOT.page.callbacks.siteTools);
		
	},
	
	backlinksClick: function(e){
		var pageId = WIKIREQUEST.info.pageId;
		var parms = new Object();
		parms['page_id'] = pageId;
		OZONE.ajax.requestModule("backlinks/BacklinksModule",parms,WIKIDOT.page.callbacks.backlinksClick);
	},
	viewSourceClick: function(e){
		var pageId = WIKIREQUEST.info.pageId;
		var parms = new Object();
		parms['page_id'] = pageId;
		OZONE.ajax.requestModule("viewsource/ViewSourceModule",parms,WIKIDOT.page.callbacks.viewSourceClick);
	},
	
	closeActionArea: function(e){
		var a = $("action-area");
		if(a){
			if(("page-options-bottom")){
				var myEffect = new fx.ScrollBottom({duration: 100, transition: fx.sineOut});
				myEffect.scrollTo("page-options-bottom");
			}
			setTimeout('$("action-area").innerHTML = "";$("action-area").style.display = "none"',200);
		}
	},
	
	userInfo: function(userId){
		var p = new Object();
		p['user_id'] = userId;
		OZONE.ajax.requestModule("users/UserInfoWinModule", p, WIKIDOT.page.callbacks.userInfo);
	},
	anonymousUserInfo: function(userString){
		var p = new Object();
		p.userString = userString;
		OZONE.ajax.requestModule("users/AnonymousInfoWinModule", p, WIKIDOT.page.callbacks.userInfo);
	},
	
	renamePage: function(e){
		var p = new Object();
		p['pageId'] =  WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("rename/RenamePageModule", p, WIKIDOT.page.callbacks.renamePage);
		
	},
	deletePage: function(e){
		var p = new Object();
		p['pageId'] =  WIKIREQUEST.info.pageId;
		p['delete'] = "yes";
		OZONE.ajax.requestModule("rename/RenamePageModule", p, WIKIDOT.page.callbacks.renamePage);
		
	},
	createPageDiscussion: function(e){
		var p = new Object();
		p['page_id'] =  WIKIREQUEST.info.pageId;
		p['action'] = "ForumAction";
		p['event'] = "createPageDiscussionThread";
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.page.callbacks.createPageDiscussion);	
	},
	
	flagPageObjectionable: function(e){
		var p = new Object();
		p.path = window.location.pathname;
		OZONE.ajax.requestModule('report/FlagPageModule', p, WIKIDOT.page.callbacks.flagPageObjectionable);
	},
	pageBugReport: function(e){
		
		OZONE.ajax.requestModule('report/BugReportModule', null, WIKIDOT.page.callbacks.pageBugReport);
	},
	pageRate: function(e){
		OZONE.ajax.requestModule('pagerate/PageRateModule', {pageId: WIKIREQUEST.info.pageId}, WIKIDOT.page.callbacks.pageRate);
	},
	parentClick: function(e){
		var p = new Object();
		p['page_id']=WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("parent/ParentPageModule",p,WIKIDOT.page.callbacks.parentClick);
		
	},
	passwordRecoveryClick: function(e){
		OZONE.ajax.requestModule("passwordrecovery/PasswordRecoveryModule", null, WIKIDOT.page.callbacks.passwordRecovery);
	},
	
	foldToc: function(e){
		var eff = new fx.Opacity($("toc-list"), {duration: 200, onComplete: function(){
			$("toc-list").style.display = "none";
			var as = $("toc-action-bar").getElementsByTagName('a');
			as[0].style.display = "none"; 
			as[1].style.display = '';
		}});
		eff.custom(1,0);
	},
	unfoldToc: function(e){
		var eff = new fx.Opacity($("toc-list"), {duration: 200});
		eff.setOpacity(0);
		$("toc-list").style.display = "block";
		eff.custom(0,1);
		var as = $("toc-action-bar").getElementsByTagName('a');
		as[1].style.display = "none";
		as[0].style.display = '';
	},
	
	search: function(e){
		var query = $("search-top-box-input").value;
		// escape query
		query = encodeURIComponent(query);
		var url = "/search:site/q/"+query;
		window.location.href=url;
		YAHOO.util.Event.preventDefault(e);
	},
	
	printClick: function(e){
		// open a new window...
		var url = '/printer--friendly/'+window.location.pathname;
		var newwindow = window.open(url, "_blank",'location=no,menubar=yes,titlebar=no,resizable=yes,scrollbars=yes,width=' + (screen.width*0.8) + ',height=' +
			(screen.height*0.8) + ',top='+ (screen.height*0.1) +',left='+(screen.width*0.1));
		return newwindow;	
	
	}
	
}

WIKIDOT.page.callbacks = {
	filesClick: function(response){
		if(!WIKIDOT.utils.handleError(response)) {return;}
		OZONE.utils.setInnerHTMLContent('action-area', response.body);	
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 200);
	},
	
	editClick: function(response){
		if(!WIKIDOT.utils.handleError(response)) {return;}
		
		if(WIKIDOT.page.vars.newPage == true){
			$('page-content').innerHTML='';
		}
		
		if(WIKIDOT.page.vars.editSectionsActive){
			WIKIDOT.page.listeners.toggleEditSections();
		}
		
		// init
		editMode = response.mode;		
		
		if(response.locked == true){
			// the page has a lock!
			WIKIDOT.page.vars.locked = true;
			// put output in the window
			OZONE.dialog.factory.shader().show();
			var cont = OZONE.dialog.factory.boxcontainer();
			cont.setContent(response.body);
			cont.showContent();
			return;
		} else {
			WIKIDOT.page.vars.locked = false;
			var pageId = WIKIREQUEST.info.pageId;
			if(pageId!=null){
				// editing old page
				if($("page-options-bottom")){
					$("page-options-bottom").style.display='none';
					$("page-options-bottom-2").style.display="none";
				}
				if($("page-options-area-bottom")){
					$("page-options-area-bottom").style.display='none';
				}
			}
			// lock information! (veeeery crucial)
			WIKIDOT.page.vars.editlock = new Object();
			WIKIDOT.page.vars.editlock.id = response['lock_id'];
			WIKIDOT.page.vars.editlock.secret = response['lock_secret'];
			WIKIDOT.page.vars.editlock.revisionId = response['page_revision_id'];
			WIKIDOT.page.vars.editlock.timeLeft = response.timeLeft;
			
		}
		
		if(editMode == 'section'){
			if(response.section == null){
				alert('Section edit error. Section does not exist');
				return;
			}
			WIKIDOT.page.vars.editlock.rangeStart = response.rangeStart;
			WIKIDOT.page.vars.editlock.rangeEnd = response.rangeEnd;
			
			// insert new div before the heading...
			var headingId = 'toc'+response.section;
			var heading = $(headingId);
			var aDiv = document.createElement('div');
			aDiv.id = 'edit-section-content';
			var pc = $("page-content");
			pc.insertBefore(aDiv, heading);
			var re = new RegExp('^h[1-'+heading.tagName.replace(/h/i,'')+']', 'i');
			var ns=heading.nextSibling;
			aDiv.appendChild(heading);
			while(ns != null){
				if(ns.tagName && ns.tagName.match(re) && ns.id.match(/^toc/)){
					break;
				}
				ns0 = ns;
				ns = ns.nextSibling;
				aDiv.appendChild(ns0);
			}	
			//also move action area below that div.
			if(ns){
				pc.insertBefore($('action-area'),ns);
			} else {
				pc.appendChild($('action-area'));
			}
		}
		
		
		
		OZONE.utils.setInnerHTMLContent('action-area', response.body);
		$("action-area").style.display = "block";
		setTimeout("OZONE.visuals.scrollTo('action-area')", 200);
		WIKIDOT.page.vars.ctrle.disable();
	},
	
	historyClick: function(response){
		if(!WIKIDOT.utils.handleError(response)) {return;}
		OZONE.utils.setInnerHTMLContent('action-area', response.body);
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
	},
	logoutClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		window.location.reload();
	},
	
	passwordRecovery: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		WIKIDOT.vars.rsakey = r.key;
		WIKIDOT.vars.loginSeed = r.seed;
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	},
	
	createAccount: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	},
	
	backlinksClick: function(response){
		if(!WIKIDOT.utils.handleError(response)) {return;}
		OZONE.utils.setInnerHTMLContent('action-area', response.body);
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	viewSourceClick: function(response){
		if(!WIKIDOT.utils.handleError(response)) {return;}
		OZONE.utils.setInnerHTMLContent('action-area', response.body);
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	
	userInfo: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.clickOutsideToClose = true;
		w.show();
		
	},
	
	renamePage: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$('action-area').innerHTML = r.body;
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	editTags: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$('action-area').innerHTML = r.body;
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	blockClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$('action-area').innerHTML = r.body;
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	pageRate: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$('action-area').innerHTML = r.body.replace(/prw54353/, 'prw54354');
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	siteTools: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$('action-area').innerHTML = r.body;
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		OZONE.dialog.hovertip.dominit("site-tools-box", {delay: 700, valign: 'center'});
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	parentClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$('action-area').innerHTML = r.body;
		$("action-area").style.display = "block";
		WIKIDOT.page.utils.addCloseToActionArea();
		setTimeout("OZONE.visuals.scrollTo('action-area')", 300);
	},
	createPageDiscussion: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		// create the URI and change page now
		var uri = "/forum/t-"+r.thread_id+'/'+r.thread_unix_title;
		window.location.href=uri;
	},
	flagPageObjectionable: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	},
	pageBugReport: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}
	
}

WIKIDOT.page.utils = {
	scrollToReference: function(elementId){
		OZONE.visuals.scrollTo(elementId, {blink: true});
	},
	
	addCloseToActionArea: function(){
		var cl = document.createElement("a");
		cl.innerHTML="close";
		cl.href="javascript:;";
		cl.className = "action-area-close"; 
		var aa = $("action-area");
		if (aa.firstChild){
			aa.insertBefore(cl,aa.firstChild);
		}else{
			aa.appendChild(cl);
		}
		YAHOO.util.Event.addListener(cl, "click", WIKIDOT.page.listeners.closeActionArea);
	},
	
	openHelpPop: function(topicName){
		// TODO: De-Wikidot.com-ize - change
		var newwindow = window.open("http://test.wikidot.com/default--screen/HelpPop/topic/"+topicName, "_blank",'location=no,menubar=no,titlebar=no,resizable=yes,scrollbars=yes,width=' + (screen.width*0.8) + ',height=' +
			(screen.height*0.8) + ',top='+ (screen.height*0.1) +',left='+(screen.width*0.1));
		return newwindow;
		
	}
}

WIKIDOT.page.fixers = {
	/** 
	 * Fix math references to show hover equations.
	 */
	 fixMathRef: function(){
	 	var erefs = YAHOO.util.Dom.getElementsByClassName("eref");
	 	var id, eref, equation;
		if(erefs && erefs.length > 0){
		 	for(var i=0; i<erefs.length; i++){
		 		eref = erefs[i];
		 		id=eref.innerHTML;
		 		equation = $("equation-"+id);
		 		if(equation){
		 			// create a tip
					var image = equation.getElementsByTagName('img').item(0).cloneNode(true);
					var text = '<b>Equation ('+id+')</b><br/><img style="margin: 1em" src="'+image.src+'"/><br/>' +
							'<span style="font-size: 90%">(click to scroll to the equation)</span>';
					OZONE.dialog.hovertip.makeTip(eref, {text: text, valign:'center', 
						style: {width: 'auto', backgroundColor: 'white'}});
					
		 		}
		 	
		 	}
		}
	 },
	 fixFootnoteRef: function(root){
	 	var frefs = YAHOO.util.Dom.getElementsByClassName("footnoteref", "a",root);
	 	for(var i=0; i<frefs.length; i++){
	 		var fref = frefs[i];
	 		var id=fref.id.replace(/^footnoteref\-/,'');
	 		var footnote = $("footnote-"+id);
	 	
	 		var content = footnote.innerHTML.replace(/<a.*?<\/a>\. /,'');
	
	 		var text = '<b>Footnote '+id+'.</b><br/>' + '<div style="margin: 0.5em 0">'+content+'</div>' +
						'<span style="font-size: 90%">(click to scroll to footnotes)</span>';
			OZONE.dialog.hovertip.makeTip(fref, {text: text, valign:'center', smartWidthLimit: 0.7,
					style: {width: 'auto', backgroundColor: 'white'}});
	 	}
	 },
	 
	 fixBibRef: function(root){
	 	var brefs = YAHOO.util.Dom.getElementsByClassName("bibcite", "a", root);
	 	for(var i=0; i<brefs.length; i++){
	 		var bref = brefs[i];
	 		var id=bref.id.replace(/bibcite\-/,'');
	 		var bibitem = $("bibitem-"+id);
	 		var content = bibitem.innerHTML.replace(/^\s*[0-9]+\.\s*/, '');
	 		var text = '<b>Reference '+id+'.</b><br/>' + '<div style="margin: 0.5em 0">'+content+'</div>' +
						'<span style="font-size: 90%">(click to scroll to bibliography)</span>';
			OZONE.dialog.hovertip.makeTip(bref, {text: text, valign:'center', smartWidthLimit: 0.7,
					style: {width: 'auto', backgroundColor: 'white'}});		
	 	}
	 	
	 },
	 fixDates: function(){
	 	OZONE.utils.formatDates();
	 },
	 /**
	  * Adds listeners to menu li elements
	  */
	 fixMenu: function(root){
	 	var r = $(root);
	 	if(r == null){return;}
	 	var els = r.getElementsByTagName("li");
	 	for (var i=0; i<els.length; i++) {
			YAHOO.util.Event.addListener(els[i], "mouseover", function(e){
				YAHOO.util.Dom.addClass(this,"sfhover");
			});
			YAHOO.util.Event.addListener(els[i], "mouseout", function(e){
				YAHOO.util.Dom.removeClass(this,"sfhover");
			});
		}
	 	
	 },
	 
	 fixEmails: function(root){
	 	var els = YAHOO.util.Dom.getElementsByClassName("wiki-email", "span",root);
	 	var el;
	 	for(var i=0; i<els.length; i++){
	 		el = els[i];
	 		if(el.innerHTML.match(/^([a-z0-9\-\.\|_])+#/i)){
				var s = el.innerHTML.split('#');
				var email = s[0].replace('|','@');
				var email2 = '';
				for(var j=email.length-1; j>=0; j--){
					email2 += email.charAt(j);
				}
				var text = s[1].replace('|','@');
				var text2 = '';
				for(var j=text.length-1; j>=0; j--){
					text2 += text.charAt(j);
				}
				var a = document.createElement('a');
				a.href='mailto:'+email2;
				a.innerHTML=text2;
				el.innerHTML = '';
				el.appendChild(a);
				el.style.visibility = "visible";
	 		}
	 	}
	 },
	 
	 fixFoldableMenus: function(root){
	 	
	 	var rootElement = $(root);
	 	if(!rootElement){
	 		return;
	 	}
	 	var divs = YAHOO.util.Dom.getElementsByClassName("foldable-list-container", 'div', rootElement);
	 	for(var i=0; i<divs.length; i++){
	 		// get all lists
	 		var uls = divs[i].getElementsByTagName('ul');
	 		for(var j=0; j<uls.length; j++){
	 			var ul = uls[j];
	 			// hide if not a direct descendant of the container
	 			var parnt = ul.parentNode;
	 			var direct = true;
	 			while(parnt && !YAHOO.util.Dom.hasClass(parnt, 'foldable-list-container')){
	 				//alert(parnt.tagName+'.'+parnt.className)
	 				if(parnt.tagName && parnt.tagName.toLowerCase() == 'li'){
	 					direct = false;
	 					break;
	 				}
	 				parnt = parnt.parentNode;
	 			}
	 			if(!direct){
	 				ul.originalDisplay = ul.style.display;
	 				ul.style.display = "none";
	 				YAHOO.util.Dom.addClass(parnt, "folded");
	 				parnt.eff = new fx.Opacity(ul, {duration: 300});
	 				// check if the li has a proper <a> element. if not - add a.
	 				var tnode = parnt.childNodes[0];
	 				if(tnode.tagName != "A"){
	 					var a = document.createElement('a');
	 					parnt.insertBefore(a, tnode);
	 					a.appendChild(tnode);
	 					a.href="javascript:;";		
	 				}
				}
	 		}
	 		
	 		// check if there is any active page here.. if so - unfold the list somehow
	 		var as = divs[i].getElementsByTagName('a');
	 		var loc = window.location.pathname;
	 		
	 		for(var j=0; j<as.length; j++){
	 			
	 			var href = as[j].href.replace(/^[a-z]*:\/\/[^\/]+\/([^\/]+).*/, '/$1');
	 			if(href == loc){
	 				var parnt = as[j].parentNode;
	 				while(parnt &&!YAHOO.util.Dom.hasClass(parnt, 'foldable-list-container')){
	 					
	 					if(parnt.tagName == 'LI' && YAHOO.util.Dom.hasClass(parnt,"folded")){
	 						YAHOO.util.Dom.replaceClass(parnt, "folded", "unfolded");
	 						var ul = parnt.getElementsByTagName('ul')[0];
	 						ul.style.display = ul.originalDisplay;
	 					}
	 					parnt = parnt.parentNode;
	 				}
	 			}
	 		}
	 		// attach a listener too
	 		YAHOO.util.Event.addListener(divs[i], "click", WIKIDOT.page.fixers._foldableMenuToggle);
	 		
	 	}
	 
	 	
	 },
	 
	 _foldableMenuToggle: function(event){
	 	var li;
	 	li = YAHOO.util.Event.getTarget(event, true);
	 	if(li.tagName == "A" && li.href != "#" && li.href != "javascript:;"){
	 		return;
	 	}
	 	while(!li.tagName || li.tagName.toLowerCase() != 'li'){
	 		li = li.parentNode;
	 	}
	 	if(!(YAHOO.util.Dom.hasClass(li, "folded") || YAHOO.util.Dom.hasClass(li, "unfolded"))){
	 		return;
	 	}
	 	
	 
	 	if(YAHOO.util.Dom.hasClass(li, "folded")){
	 		// unfold
	 		YAHOO.util.Dom.replaceClass(li, "folded", "unfolded");
	 		var ul = li.getElementsByTagName('ul')[0];
	 		li.eff.setOpacity(0);
	 		ul.style.display = ul.originalDisplay;
	 		li.eff.custom(0,1);
	 	}else{
	 		// fold
	 		YAHOO.util.Dom.replaceClass(li, "unfolded", "folded");
	 		var ul = li.getElementsByTagName('ul')[0];
	 		
	 		ul.style.display = 'none';
	 	}
	 },
	 /** 
	  * Inserts A elements into LI elements if not present.
	  */
	 fixMenuList: function(rootElement){
	 	rootElement = $(rootElement);
	 	if(!rootElement){
	 		return;
	 	}
	 	var lis = rootElement.getElementsByTagName('li');
	 	for(var i=0; i<lis.length; i++){
	 		var tnode = lis[i].childNodes[0];
	 		if(tnode.tagName != "A" && tnode.nodeType == 3 && tnode.innerHTML != ""){
	 			var a = document.createElement('a');
	 			lis[i].insertBefore(a, tnode);
	 			a.appendChild(tnode);
	 			a.href="javascript:;";		
	 		}
	 	}
	 }
	
}

WIKIDOT.page.vars = {
	editSectionsActive: false
	
}

WIKIDOT.page.account = {};
WIKIDOT.page.account.shower = function(e){
	// the listener to show account options
	var ao = $("account-options");
	if(!ao.eff){
		ao.eff = new fx.Opacity(ao, {duration:200});
		
	}
	ao.eff.setOpacity(0);
	ao.style.display="block";
	ao.eff.custom(0,1);
}
WIKIDOT.page.account.closer = function(e){
	var ao = $("account-options");
	var rt = YAHOO.util.Event.getRelatedTarget(e);
	// check if rt is ao or is a child of ao
	var is = false;
	if(rt == ao) is = true;
	if(rt.parentNode == ao) is=true;
	if(rt.parentNode.parentNode == ao) is=true;
	if(rt.parentNode.parentNode.parentNode == ao) is=true;
	if(is == true) return;
	ao.eff.setOpacity(0);
	ao.style.display = "none";
}

/* initialize a few things ;-) */
WIKIDOT.page.init = function(){
	YAHOO.util.Event.addListener("edit-button", "click", WIKIDOT.page.listeners.editClick);
	YAHOO.util.Event.addListener("pagerate-button", "click", WIKIDOT.page.listeners.pageRate);
	YAHOO.util.Event.addListener("tags-button", "click", WIKIDOT.page.listeners.editTags);
	YAHOO.util.Event.addListener("history-button", "click", WIKIDOT.page.listeners.historyClick);
	YAHOO.util.Event.addListener("files-button", "click", WIKIDOT.page.listeners.filesClick);
	YAHOO.util.Event.addListener("print-button", "click", WIKIDOT.page.listeners.printClick);
	YAHOO.util.Event.addListener("site-tools-button", "click", WIKIDOT.page.listeners.siteTools);
	YAHOO.util.Event.addListener("more-options-button", "click", WIKIDOT.page.listeners.moreOptionsClick);
	
	YAHOO.util.Event.addListener("edit-append-button", "click", WIKIDOT.page.listeners.append);
	YAHOO.util.Event.addListener("edit-sections-button", "click", WIKIDOT.page.listeners.toggleEditSections);
	YAHOO.util.Event.addListener("backlinks-button", "click", WIKIDOT.page.listeners.backlinksClick);
	YAHOO.util.Event.addListener("parent-page-button", "click", WIKIDOT.page.listeners.parentClick);
	
	YAHOO.util.Event.addListener("view-source-button", "click", WIKIDOT.page.listeners.viewSourceClick);
	YAHOO.util.Event.addListener("page-block-button", "click", WIKIDOT.page.listeners.blockClick);
	YAHOO.util.Event.addListener("rename-move-button", "click", WIKIDOT.page.listeners.renamePage);
	YAHOO.util.Event.addListener("delete-button", "click", WIKIDOT.page.listeners.deletePage);
	
	YAHOO.util.Event.addListener("search-top-box-form", "submit", WIKIDOT.page.listeners.search);
	
	
	OZONE.dom.onDomReady(function(){
		OZONE.dialog.hovertip.dominit("html-body", {delay: 700, valign: 'center'});
		WIKIDOT.page.fixers.fixMenuList("top-bar");
		WIKIDOT.page.fixers.fixFoldableMenus("side-bar");
		WIKIDOT.page.fixers.fixMathRef();
		WIKIDOT.page.fixers.fixFootnoteRef();
		WIKIDOT.page.fixers.fixBibRef();
		WIKIDOT.page.fixers.fixDates($("html-body"));
		WIKIDOT.page.fixers.fixEmails($("page-content"));
		WIKIDOT.render.fixAvatarHover();
		
		var accountButton = $("account-topbutton");
		if(accountButton){
			YAHOO.util.Event.addListener(accountButton, "mousedown", WIKIDOT.page.account.shower);
			YAHOO.util.Event.addListener("account-options", "mouseout", WIKIDOT.page.account.closer);
		}
			WIKIDOT.page.fixers.fixMenu("top-bar");
			WIKIDOT.page.fixers.fixMenu("side-bar");
		
		OZONE.visuals.initScroll();
		var not = $("notifications-dialog");
		if(not != null){
			var w = new OZONE.dialogs.Dialog();
			w.content = not.innerHTML;
			w.show();
			setTimeout("OZONE.dialog.factory.boxcontainer().centerContent();", 1000);
		}
		
		// check if to start page editing now
		var path = window.location.pathname;
		if(path.match(/^\/[a-z0-9\-:]+\/edit\/true/)){
			WIKIDOT.page.listeners.editClick();
				// force use the template
				
		}
		// check if need highlighting
		/* 
		var path = window.location.pathname;
		if(path.match(/^\/[^\/]+.*?\/highlight\/([^\/]+)(\/|$)/)){
			var htext = path.replace(/^\/[^\/]+.*?\/highlight\/([^\/]+)(\/|$)/, "$1");
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
	
	var ldates = {
		ago: "temu", day: "dzień", days: "dni", hours: "godziny", hour: "godzina", minutes: "minuty", minute: "minuta", seconds: 'sekundy', second: 'sekunda'
	}

	OZONE.loc.addMessages(ldates, "pl");
	
	
	// attache ctrl+e for editing
	// handle ctrl+s
	var ctrle = new YAHOO.util.KeyListener(document, {keys:69, ctrl:true}, function(type,e){
			e = e[1];
			WIKIDOT.page.listeners.editClick(e);
			YAHOO.util.Event.stopEvent(e);
		}
	);
	ctrle.enable();
	WIKIDOT.page.vars.ctrle = ctrle;
}


WIKIDOT.page.init();
