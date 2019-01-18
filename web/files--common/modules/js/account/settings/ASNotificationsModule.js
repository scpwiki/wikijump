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

WIKIDOT.modules.ASNotificationsModule = {};

WIKIDOT.modules.ASNotificationsModule.listeners = {
	saveReceiveDigest: function(e){
		var receive = $("as-receive-digest").checked;
		var p = new Object();
		
		p.action = "AccountSettingsAction";
		p.event = "saveReceiveDigest";
		if(receive){
			p.receive = "yes";
		}
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASNotificationsModule.callbacks.saveReceiveDigest);
	},
	saveReceiveNewsletter: function(e){
		var receive = $("as-receive-newsletter").checked;
		var p = new Object();
		
		p.action = "AccountSettingsAction";
		p.event = "saveReceiveNewsletter";
		if(receive){
			p.receive = "yes";
		}
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASNotificationsModule.callbacks.saveReceiveDigest);
	}		
}

WIKIDOT.modules.ASNotificationsModule.callbacks = {
	saveReceiveDigest: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
		
}
