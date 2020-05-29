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

WIKIDOT.modules.AccountAdminOfModule = {};

WIKIDOT.modules.AccountAdminOfModule.vars = {
	currentSiteId: null
}

WIKIDOT.modules.AccountAdminOfModule.listeners = {
	resign: function(e, siteId, siteName){
		WIKIDOT.modules.AccountAdminOfModule.vars.currentSiteId = siteId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("admin-resign-dialog").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, resign'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, resign', WIKIDOT.modules.AccountAdminOfModule.listeners.resign2);
		w.show();
	},
	
	resign2: function(e){
		var siteId = WIKIDOT.modules.AccountAdminOfModule.vars.currentSiteId;
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'adminResign';
		p.site_id = siteId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountAdminOfModule.callbacks.resign );
	}
}

WIKIDOT.modules.AccountAdminOfModule.callbacks = {
	resign: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "You are no longer an admin of this site.";
			w.show();
			WIKIDOT.modules.AccountModule.utils.loadModule('am-adminof');
		} else {
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			w.show();
			
		}
	}	
}