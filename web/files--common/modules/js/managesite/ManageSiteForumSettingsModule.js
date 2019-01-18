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

WIKIDOT.modules.ManageSiteForumSettingsModule = {};

WIKIDOT.modules.ManageSiteForumSettingsModule.listeners = {
	activateForum: function(e){
		var p = new Object();
		p.action = "ManageSiteForumAction";
		p.event = "activateForum";
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteForumSettingsModule.callbacks.activateForum);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Initializing forum...";
		w.show();
	},
	
	saveNesting: function(e){
		var nest = $("max-nest-level").value;
		var p = new Object();
		p['action'] = "ManageSiteForumAction";
		p['event'] = "saveForumDefaultNesting";
		p['max_nest_level'] = nest;
		OZONE.ajax.requestModule("Empty", p,WIKIDOT.modules.ManageSiteForumSettingsModule.callbacks.saveNesting);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}	
}

WIKIDOT.modules.ManageSiteForumSettingsModule.callbacks = {
	saveNesting: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved.";
		w.show();
	},
	activateForum: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Forum has been activated.";
		w.show();
		setTimeout("WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-forum-settings')", 1000);
	}	
}