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

WIKIDOT.modules.UserInfoWinModule = {};

WIKIDOT.modules.UserInfoWinModule.listeners = {
	flagUser: function(e, userId){
		OZONE.ajax.requestModule('report/FlagUserModule', {targetUserId: userId}, WIKIDOT.modules.UserInfoWinModule.callbacks.flagUser);
		
	},
	addContact: function(e, userId){
		OZONE.ajax.requestModule('userinfo/UserAddToContactsModule', {userId: userId}, WIKIDOT.modules.UserInfoWinModule.callbacks.addContact);
	}
	
}

WIKIDOT.modules.UserInfoWinModule.callbacks = {
	flagUser: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();		
	},
	addContact: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();	
	}
}