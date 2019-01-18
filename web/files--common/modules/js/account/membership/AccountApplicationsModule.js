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

WIKIDOT.modules.AccountApplicationsModule = {};

WIKIDOT.modules.AccountApplicationsModule.vars = {
	currentSiteId: null
}

WIKIDOT.modules.AccountApplicationsModule.listeners = {
	remove: function(e, siteId, siteName){
		WIKIDOT.modules.AccountApplicationsModule.vars.currentSiteId = siteId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("application-remove-dialog").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, remove'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove', WIKIDOT.modules.AccountApplicationsModule.listeners.remove2);
		w.show();
	},
	
	remove2: function(e,siteId0, siteName0){
		if(typeof(siteId0) != 'number'){
			var siteId = WIKIDOT.modules.AccountApplicationsModule.vars.currentSiteId;
		}else{
			var siteId = siteId0;
		}
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'removeApplication';
		p.site_id = siteId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountApplicationsModule.callbacks.remove );
	}
}

WIKIDOT.modules.AccountApplicationsModule.callbacks = {
	remove: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The application has been removed.";
		w.show();
		WIKIDOT.modules.AccountModule.utils.loadModule('am-applications');
		
	}	
}