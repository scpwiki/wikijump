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

WIKIDOT.modules.ManagerSiteDeleteModule = {};

WIKIDOT.modules.ManagerSiteDeleteModule.vars = {
	currentCategory: null
}

WIKIDOT.modules.ManagerSiteDeleteModule.listeners = {
	deleteSite: function(event){
		var p = new Object();
		
		OZONE.ajax.requestModule("managesite/ManageSiteDelete2Module", p, WIKIDOT.modules.ManagerSiteDeleteModule.callbacks.deleteSite);
	},
	deleteSite2: function(event){
		var p = new Object();
		p.action = "ManageSiteAction";
		p.event = "DeleteSite";
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManagerSiteDeleteModule.callbacks.deleteSite2);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Deleting the site...";
		w.show();
	}		
}

WIKIDOT.modules.ManagerSiteDeleteModule.callbacks = {
	deleteSite: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("sm-delete-box").innerHTML = r.body;
		
	},
	deleteSite2: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The site has been deleted.";
		setTimeout('window.location.href="http://www.'+URL_DOMAIN+'/account:you/start/deletedsites"', 1000);
		w.show();	
	}
}
