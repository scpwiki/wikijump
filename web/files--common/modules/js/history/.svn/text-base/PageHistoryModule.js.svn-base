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

WIKIDOT.modules.PageHistoryModule = {};

WIKIDOT.modules.PageHistoryModule.vars = {};

WIKIDOT.modules.PageHistoryModule.listeners = {
	
	updateList: function(e){
		var p = new Object();
		p['page'] = 1;
		p['perpage'] = $("h-perpage").value;
		p['page_id'] = WIKIREQUEST.info.pageId;
		
		// which revisions...
		var o = new Object();
		if($("rev-type-all").checked) o.all = true;
		if($("rev-type-source").checked) o.source = true;
		if($("rev-type-title").checked) o.title = true;
		if($("rev-type-move").checked) o.move = true;
		if($("rev-type-files").checked) o.files = true;
		if($("rev-type-meta").checked) o.meta = true;
		
		p['options'] = JSON.stringify(o);
		
		WIKIDOT.modules.PageHistoryModule.vars.params = p; // for pagination
		
		OZONE.ajax.requestModule("history/PageRevisionListModule", p, WIKIDOT.modules.PageHistoryModule.callbacks.updateList);
	},
	
	compareClick: function(e){
		
		var form1 = $("history-form-1");
		var radios = form1.getElementsByTagName('input');
		for (i=0;i<radios.length;i++){
			if(radios[i].type=='radio' && radios[i].name=='from' && radios[i].checked){
				selected_from=radios[i].value;
			}
			if(radios[i].type=='radio' && radios[i].name=='to' && radios[i].checked){
				selected_to=radios[i].value;
			}
			if(radios[i].type=='radio' && radios[i].name=='show-type' && radios[i].checked){
				show_type=radios[i].value;
			}
		}
		var parms = new Object();
		parms['from_revision_id']=selected_from;
		parms['to_revision_id']=selected_to;
		parms['show_type'] = 'inline';
		OZONE.ajax.requestModule("history/PageDiffModule",parms,WIKIDOT.modules.PageHistoryModule.callbacks.compareClick);
	},
	
	closeActionArea: function(e){
		var a = $("history-subarea");
		a.innerHTML = "";
		a.style.display="none";
		
	}, 
	
	revert: function(e, revisionId){
		
		var id = "revision-row-"+revisionId;
		var revisionNumber = $(id).getElementsByTagName('td')[0].innerHTML;
		
		//show a confirm dialog first
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = "<h1>Revert page revision?</h1><p>Are you sure you want to revert page version to " +
				"revision number <strong>"+revisionNumber+"</strong>?</p><p>" +
				"This would affect only the title of the page and/or source but not any files " +
				"attachment, page name or other metadata.</p>" +
				"<p>If you say \"yes\" a new revision will be created and title and source of " +
				"the revision "+revisionNumber+" will be copied.</p>";
		w.buttons = ["cancel", "yes, revert"];
		w.addButtonListener("cancel", w.close);
		w.addButtonListener("yes, revert", WIKIDOT.modules.PageHistoryModule.listeners.revert2);
		w.show();
		WIKIDOT.modules.PageHistoryModule.vars.revertRevisionId = revisionId;
	},
	
	revert2: function(e, force){
		var p = new Object();
		p['pageId'] =  WIKIREQUEST.info.pageId;
		p['revisionId'] = WIKIDOT.modules.PageHistoryModule.vars.revertRevisionId;
		p['action'] = "WikiPageAction";
		p['event'] = "revert";
		if(force == true){
			p['force'] = "yes";
		}
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PageHistoryModule.callbacks.revert);
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Reverting page version...";
		w.show();
		
	},
	watchPage: function(e){
		var p = new Object();
		p.pageId = WIKIREQUEST.info.pageId;
		p.action = "WatchAction";
		p.event = "watchPage";
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PageHistoryModule.callbacks.watchPage);
	}
		
}

WIKIDOT.modules.PageHistoryModule.callbacks = {
	
	updateList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("revision-list").innerHTML = r.body;
		setTimeout("OZONE.visuals.scrollTo('action-area')", 100);
		OZONE.dialog.hovertip.makeTip($('action-area').getElementsByTagName('a'),
				{style: {width: 'auto'}});
		OZONE.dialog.hovertip.makeTip($('action-area').getElementsByTagName('span'),
				{style: {width: 'auto'}});
		OZONE.utils.formatDates("revision-list");

	},
	updatePageList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("revision-list").innerHTML = r.body;
		OZONE.dialog.hovertip.makeTip($('action-area').getElementsByTagName('a'),
				{style: {width: 'auto'}});
		OZONE.dialog.hovertip.makeTip($('action-area').getElementsByTagName('span'),
				{style: {width: 'auto'}});
		OZONE.utils.formatDates("revision-list");

	},
	
	compareClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent('history-subarea', r.body);
		WIKIDOT.modules.PageHistoryModule.utils.addCloseToActionArea();
		area = $('history-subarea');
		area.style.display="block";
		setTimeout("OZONE.visuals.scrollTo('history-subarea')", 100);
		OZONE.utils.formatDates('history-subarea');
	},
	
	showVersionClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		$('page-content').innerHTML = r.body;
		OZONE.utils.formatDates('page-content');
		var el = $('page-title');
		if(el){el.innerHTML = r.title;}
		setTimeout("OZONE.visuals.scrollTo('header')", 50);
	},
	
	showSource: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("history-subarea", r.body);
		$("history-subarea").style.display="block";
		setTimeout("OZONE.visuals.scrollTo('history-subarea')", 100);
		WIKIDOT.modules.PageHistoryModule.utils.addCloseToActionArea();
		//newwindow = window.open(null, "_blank",'location=no,menubar=no,titlebar=no,resizable=yes,scrollbars=yes,width=' + (screen.width*0.8) + ',height=' +
	},
	
	revert: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		if(r.locks == true){
			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.show();
			return;
		}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Page content has been reverted.";
		w.show();
		setTimeout('window.location.href="/'+WIKIREQUEST.info.requestPageName+'"',1000);
		
	},
	watchPage: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Page added to watched.";
		w.show();
	}

}

WIKIDOT.modules.PageHistoryModule.utils = {
	addCloseToActionArea: function(){
		var cl = document.createElement("a");
		cl.innerHTML="close";
		cl.href="javascript:;";
		cl.className = "action-area-close"; 
		var aa = $("history-subarea");
		if (aa.firstChild){
			aa.insertBefore(cl,aa.firstChild);
		}else{
			aa.appendChild(cl);
		}
		YAHOO.util.Event.addListener(cl, "click", WIKIDOT.modules.PageHistoryModule.listeners.closeActionArea);
	}
}

// nasty _small_ function to show archive versions of the page
function showVersion(revisionId){
	var parms = new Object();
	parms['revision_id'] = revisionId;
	OZONE.ajax.requestModule("history/PageVersionModule",parms,WIKIDOT.modules.PageHistoryModule.callbacks.showVersionClick);
}

/**
 * Shows page source for the given revision.
 */
function showSource(revisionId){
	//newwindow = window.open(null, "_blank",'location=no,menubar=no,titlebar=no,resizable=yes,scrollbars=yes,width=' + (screen.width*0.8) + ',height=' +

	var parms = new Array();
	parms['revision_id'] = revisionId;
	OZONE.ajax.requestModule("history/PageSourceModule",parms,WIKIDOT.modules.PageHistoryModule.callbacks.showSource);
}

/**
 * Make the selected revision the current one.
 */
function revertTo(revisionId){
	confirm("revision???");		

}

function updatePagedList(pageNo){
	var p = WIKIDOT.modules.PageHistoryModule.vars.params;
	p['page'] = pageNo;
	OZONE.ajax.requestModule("history/PageRevisionListModule", p, WIKIDOT.modules.PageHistoryModule.callbacks.updatePageList);
	
}

// bind!
YAHOO.util.Event.addListener("history-compare-button", "click", WIKIDOT.modules.PageHistoryModule.listeners.compareClick);
WIKIDOT.modules.PageHistoryModule.listeners.updateList(null);
