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

WIKIDOT.modules.SiteToolsModule = {};

WIKIDOT.modules.SiteToolsModule.listeners = {
	wantedPages: function(e){
		OZONE.ajax.requestModule("sitetools/WantedPagesModule", null, WIKIDOT.modules.SiteToolsModule.callbacks.setContent);
	},
	orphanedPages: function(e){
		OZONE.ajax.requestModule("sitetools/OrphanedPagesModule", null, WIKIDOT.modules.SiteToolsModule.callbacks.setContent);
	}		
}

WIKIDOT.modules.SiteToolsModule.callbacks = {
	setContent: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("st-action-area").innerHTML = r.body;
		OZONE.visuals.scrollTo("st-action-area");
	}	
}