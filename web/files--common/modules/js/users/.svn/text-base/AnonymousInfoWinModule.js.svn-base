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

WIKIDOT.modules.AnonymousInfoWinModule = {};

WIKIDOT.modules.AnonymousInfoWinModule.listeners = {
	flagAnonymous: function(e, userString){
		OZONE.ajax.requestModule('report/FlagAnonymousModule', {userString: userString}, WIKIDOT.modules.AnonymousInfoWinModule.callbacks.flagAnonymous);
		
	}
}

WIKIDOT.modules.AnonymousInfoWinModule.callbacks = {
	flagAnonymous: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();		
	}	
}
