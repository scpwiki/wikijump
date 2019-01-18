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

WIKIDOT.modules.ManageSitePageAbuseModule = {};

WIKIDOT.modules.ManageSitePageAbuseModule.listeners = {
	clear: function(e, path){
		var p = new Object();
		p.action = "ManageSiteAbuseAction";
		p.event = "clearPageFlags";
		p.path = path;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSitePageAbuseModule.callbacks.clear);
	}	
	
}

WIKIDOT.modules.ManageSitePageAbuseModule.callbacks = {
	clear: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Flags cleared";
		w.show();
		
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule("sm-abuse-page");
	}	
}