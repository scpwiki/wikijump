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

WIKIDOT.modules.ASInvitationsModule = {};

WIKIDOT.modules.ASInvitationsModule.listeners = {
	save: function(e){
		var val = $("receive-invitations-ch").checked;
		var p = new Object();
		if(val){
			p.receive = true;
		}
		p.action = "AccountSettingsAction";
		p.event = "saveReceiveInvitations";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASInvitationsModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving preferences...";
		w.show();
	}	
}

WIKIDOT.modules.ASInvitationsModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Preferences saved."	;
		w.show();	
	
	}
		
}