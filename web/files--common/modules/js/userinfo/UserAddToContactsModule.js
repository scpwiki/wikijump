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

WIKIDOT.modules.UserAddToContacts = {};

WIKIDOT.modules.UserAddToContacts.listeners = {
	addContact: function(event, userId)	{
		var p = new Object();
		p.action = "ContactsAction";
		p.event = "addContact";
		p.userId = userId;
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.UserAddToContacts.callbacks.addContact);
	}
}

WIKIDOT.modules.UserAddToContacts.callbacks = {
	addContact: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User added to contacts";
		w.show();
		if(WIKIDOT.modules.AccountContactsModule){
			WIKIDOT.modules.AccountContactsModule.listeners.refresh();
		}
	}
	
}