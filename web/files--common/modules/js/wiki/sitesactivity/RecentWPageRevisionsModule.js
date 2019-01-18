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

WIKIDOT.modules.RecentWPageRevisionsModule = {};

WIKIDOT.modules.RecentWPageRevisionsModule.vars = {};

WIKIDOT.modules.RecentWPageRevisionsModule.listeners = {
	update: function(){
		OZONE.ajax.requestModule('wiki/sitesactivity/RecentWPageRevisionsModule', null, WIKIDOT.modules.RecentWPageRevisionsModule.callbacks.update);
	}	
}

WIKIDOT.modules.RecentWPageRevisionsModule.callbacks = {
	update: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var body = r.body.replace(/<div[^>]*>/, '').replace(/<\/div>\s*$/, '');
		
		if(body != $("recent-w-page-revisions").innerHTML){

			$("recent-w-page-revisions").innerHTML = body;
			
//			$("recent-w-page-revisions")

		}
	}	
}

WIKIDOT.modules.RecentWPageRevisionsModule.init = function(){
	setTimeout('WIKIDOT.modules.RecentWPageRevisionsModule.listeners.update()', 20000);
}

//WIKIDOT.modules.RecentWPageRevisionsModule.init();
