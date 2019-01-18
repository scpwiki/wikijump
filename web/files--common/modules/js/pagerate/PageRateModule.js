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

WIKIDOT.modules.PageRateModule = {};

WIKIDOT.modules.PageRateModule.listeners = {
	showWho: function(e, pageId){
		var p = new Object();
		p.pageId = WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("pagerate/WhoRatedPageModule", p, WIKIDOT.modules.PageRateModule.callbacks.showWho);
	}	
}

WIKIDOT.modules.PageRateModule.callbacks = {
	showWho: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("who-rated-page-area").innerHTML = r.body;
		OZONE.visuals.scrollTo($("who-rated-page-area"));
		WIKIDOT.render.fixAvatarHover($("who-rated-page-area"));
	}	
}