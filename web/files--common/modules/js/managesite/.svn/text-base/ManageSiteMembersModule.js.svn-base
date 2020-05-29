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

WIKIDOT.modules.ManagerSiteMembersModule = {};

WIKIDOT.modules.ManagerSiteMembersModule.listeners = {
	save: function(e){
		var parms = OZONE.utils.formToArray("sm-mem-form");
		parms['action'] = "ManageSiteMembershipAction";
		parms['event'] = "saveMemberPolicy";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManagerSiteMembersModule.callbacks.save);	
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	
	},
	cancel: function(e){
		OZONE.ajax.requestModule("managesite/ManageSiteModule", null, WIKIDOT.modules.ManagerSiteMembersModule.callbacks.cancel)
	}
}

WIKIDOT.modules.ManagerSiteMembersModule.callbacks = {
	save: function(){
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	},
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	}
	
}

WIKIDOT.modules.ManagerSiteMembersModule.init = function(){
	YAHOO.util.Event.addListener("sm-members-cancel", "click", WIKIDOT.modules.ManagerSiteMembersModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-members-save", "click", WIKIDOT.modules.ManagerSiteMembersModule.listeners.save);

}
	
WIKIDOT.modules.ManagerSiteMembersModule.init();
