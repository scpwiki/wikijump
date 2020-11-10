

Wikijump.modules.ManageSiteBackupModule = {};

Wikijump.modules.ManageSiteBackupModule.listeners = {
	requestBackup: function(e){
		var p = OZONE.utils.formToArray("backup-form");
		p.action = "ManageSiteBackupAction";
		p.event = "requestBackup";

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteBackupModule.callbacks.requestBackup);
	},

	deleteBackup: function(e){
		var p = OZONE.utils.formToArray("backup-form");
		p.action = "ManageSiteBackupAction";
		p.event = "deleteBackup";

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteBackupModule.callbacks.deleteBackup);

	}
}

Wikijump.modules.ManageSiteBackupModule.callbacks = {
	requestBackup: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		// ok, reload the module now.
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-backup');
		OZONE.visuals.scrollTo('header');
	},

	deleteBackup: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		// ok, reload the module now.
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-backup');
		OZONE.visuals.scrollTo('header');
	}
}
