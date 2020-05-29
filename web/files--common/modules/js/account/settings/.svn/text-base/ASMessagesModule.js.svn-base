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

WIKIDOT.modules.ASMessagesModule = {};

WIKIDOT.modules.ASMessagesModule.listeners = {
	save: function(e){
		
		var p = OZONE.utils.formToArray("receive-pl-form");
	
		p.action = "AccountSettingsAction";
		p.event = "saveReceiveMessages";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASMessagesModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving preferences...";
		w.show();
	}	
}

WIKIDOT.modules.ASMessagesModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Preferences saved."	;
		w.show();	
	
	}
		
}