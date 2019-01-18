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

WIKIDOT.modules.ManageSiteLetUsersInviteModule = {}

WIKIDOT.modules.ManageSiteLetUsersInviteModule.vars = {
	randoms: new Object(),
	services: null
}

WIKIDOT.modules.ManageSiteLetUsersInviteModule.listeners = {
	
	save: function(e){
		var p = new Object();
		p.action = "ManageSiteMembershipAction";
		p.event = "letUsersInviteSave";
		p.enableLetUsersInvite = $("sm-allow-users-invite").checked;

		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteLetUsersInviteModule.callbacks.save);
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}	
	
}

WIKIDOT.modules.ManageSiteLetUsersInviteModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
}
WIKIDOT.modules.ManageSiteLetUsersInviteModule.init = function(){
	
}

WIKIDOT.modules.ManageSiteLetUsersInviteModule.init();
