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

WIKIDOT.modules.ManagerSiteRenameModule = {};

WIKIDOT.modules.ManagerSiteRenameModule.vars = {
	currentCategory: null
}

WIKIDOT.modules.ManagerSiteRenameModule.listeners = {
	renameSite: function(event){
		var p = new Object();
		p.unixName = $("sm-rename-site-unixname").value;
		p.action = 'ManageSiteAction';
		p.event = 'renameSite';
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManagerSiteRenameModule.callbacks.renameSite);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Changing the URL...";
		w.show();
	}	
}

WIKIDOT.modules.ManagerSiteRenameModule.callbacks = {
	renameSite: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The URL has been changed.";
		w.show();

		setTimeout('window.location.href="http://'+r.unixName+'.'+URL_DOMAIN+'"', 500);
		
	}
	
}
