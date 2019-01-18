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

WIKIDOT.modules.AccountMemberOfModule = {};

WIKIDOT.modules.AccountMemberOfModule.vars = {
	signOffId : null
}

WIKIDOT.modules.AccountMemberOfModule.listeners = {
	signOff: function(e, siteInfo){
		WIKIDOT.modules.AccountMemberOfModule.vars.signOffId = siteInfo[0];
		var siteName = siteInfo[1];
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("signoff-window").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, sign me off'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, sign me off', WIKIDOT.modules.AccountMemberOfModule.listeners.signOff2);
		w.show();
	},
	
	signOff2: function(e){
		var siteId = WIKIDOT.modules.AccountMemberOfModule.vars.signOffId;
		var p = new Object();
		p.site_id = siteId;
		p.action = 'AccountMembershipAction';
		p.event = 'signOff';
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountMemberOfModule.callbacks.signOff);
	}	
	
}

WIKIDOT.modules.AccountMemberOfModule.callbacks = {
	signOff: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "You have successfully signed off from this site";
			w.show();
			WIKIDOT.modules.AccountModule.utils.loadModule("am-memberof");
		}else{
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			w.show();
		}
		
	}	
}