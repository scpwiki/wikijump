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

WIKIDOT.modules.ManageSiteSecureAccessModule = {}

WIKIDOT.modules.ManageSiteSecureAccessModule.listeners = {
	save: function(e){
		var p = new Object();
		p.action = "ManageSiteAction";
		p.event = "saveSecureAccess";
		p.secureMode = $("sm-ssl-mode-select").value;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteSecureAccessModule.callbacks.save);
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}	
	
}

WIKIDOT.modules.ManageSiteSecureAccessModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
		
		// reload the page!
		window.location.reload();
	}
}
