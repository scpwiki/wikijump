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

WIKIDOT.modules.CreateAccountModule = {};

WIKIDOT.modules.CreateAccountModule.vars = {};

WIKIDOT.modules.CreateAccountModule.listeners = {
	createClick: function(e){
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", null,WIKIDOT.modules.CreateAccountModule.callbacks.createClick );
	},
	cancel: function(e){
		var p = new Object();
		p.action = "CreateAccountAction";
		p.event = "cancel";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.CreateAccountModule.callbacks.cancel);
	}
}

WIKIDOT.modules.CreateAccountModule.callbacks = {
	createClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
		
		// store seed and key
		WIKIDOT.modules.CreateAccountModule.vars.rsakey = r.key;
		
	},
	cancel: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.dialog.cleanAll();
	}
}
