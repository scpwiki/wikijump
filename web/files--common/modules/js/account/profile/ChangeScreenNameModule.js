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

WIKIDOT.modules.ChangeScreenNameModule = {};

WIKIDOT.modules.ChangeScreenNameModule.listeners = {
	save: function(e){
		var p = {};
		p['action'] = "AccountProfileAction";
		p['event'] = "changeScreenName";
		p.screenName = $("ap-screen-name-input").value;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ChangeScreenNameModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Changing the screen name...";
		w.show();
		YAHOO.util.Event.stopEvent(e);
	}
}

WIKIDOT.modules.ChangeScreenNameModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your screen name has been changed!";
		w.show();

		OZONE.ajax.requestModule('account/profile/ChangeScreenNameModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick);

	}
}

