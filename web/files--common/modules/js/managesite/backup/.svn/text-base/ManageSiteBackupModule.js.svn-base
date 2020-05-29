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

WIKIDOT.modules.ManageSiteBackupModule = {};

WIKIDOT.modules.ManageSiteBackupModule.listeners = {
	requestBackup: function(e){
		var p = OZONE.utils.formToArray("backup-form");
		p.action = "ManageSiteBackupAction";
		p.event = "requestBackup";

		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteBackupModule.callbacks.requestBackup);
	},
	
	deleteBackup: function(e){
		var p = OZONE.utils.formToArray("backup-form");
		p.action = "ManageSiteBackupAction";
		p.event = "deleteBackup";

		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteBackupModule.callbacks.deleteBackup);
		
	}
}

WIKIDOT.modules.ManageSiteBackupModule.callbacks = {
	requestBackup: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		// ok, reload the module now.
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-backup');
		OZONE.visuals.scrollTo('header');	
	},
	
	deleteBackup: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		// ok, reload the module now.
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-backup');
		OZONE.visuals.scrollTo('header');	
	}
}
