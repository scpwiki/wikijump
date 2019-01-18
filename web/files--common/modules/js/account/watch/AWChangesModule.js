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

WIKIDOT.modules.AWChangesModule = {};

WIKIDOT.modules.AWChangesModule.listeners = {
	showWatchedPages: function(e){
		OZONE.ajax.requestModule("/account/watch/AWPagesListModule", null, WIKIDOT.modules.AWChangesModule.callbacks.showWatchedPages);
	},
	hideWatchedPages: function(e){
		$("watched-pages-list").innerHTML="";
		$("watched-pages-list").style.display = "none";
		$("show-watched-pages-button").style.display="";
		$("hide-watched-pages-button").style.display="none";
	},
	
	removeWatchedPage: function(e, pageId){
		var p = new Object();
		p.pageId = pageId;
		p.action = "WatchAction";
		p.event = "removeWatchedPage";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AWChangesModule.callbacks.removeWatchedPage);
	},
	updateList: function(event,pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}

		OZONE.ajax.requestModule("account/watch/AWChangesListModule", p, WIKIDOT.modules.AWChangesModule.callbacks.updateList);
	}	
}

WIKIDOT.modules.AWChangesModule.callbacks = {
	showWatchedPages: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var el = $("watched-pages-list");
		el.innerHTML = r.body;
		el.style.display = "block";
		$("hide-watched-pages-button").style.display="";
		$("show-watched-pages-button").style.display="none";
	},
	
	removeWatchedPage: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Page not being watched any more.";
		w.show();
		WIKIDOT.modules.AWChangesModule.listeners.showWatchedPages();	
		WIKIDOT.modules.AWChangesModule.listeners.updateList();
	},
	updateList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		$("watched-changes-list").innerHTML = r.body;
		OZONE.utils.formatDates("watched-changes-list");
		OZONE.dialog.hovertip.makeTip($("watched-changes-list").getElementsByTagName('span'),
				{style: {width: 'auto'}});
	}

}

WIKIDOT.modules.AWChangesModule.init = function(){
		OZONE.utils.formatDates("watched-changes-list");
		OZONE.dialog.hovertip.makeTip($("watched-changes-list").getElementsByTagName('span'),
				{style: {width: 'auto'}});
}

WIKIDOT.modules.AWChangesModule.init();
