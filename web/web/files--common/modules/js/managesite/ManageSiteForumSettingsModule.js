

Wikijump.modules.ManageSiteForumSettingsModule = {};

Wikijump.modules.ManageSiteForumSettingsModule.listeners = {
	activateForum: function(e){
		var p = new Object();
		p.action = "ManageSiteForumAction";
		p.event = "activateForum";

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteForumSettingsModule.callbacks.activateForum);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Initializing forum...";
		w.show();
	},

	saveNesting: function(e){
		var nest = $("max-nest-level").value;
		var p = new Object();
		p['action'] = "ManageSiteForumAction";
		p['event'] = "saveForumDefaultNesting";
		p['max_nest_level'] = nest;
		OZONE.ajax.requestModule("Empty", p,Wikijump.modules.ManageSiteForumSettingsModule.callbacks.saveNesting);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
}

Wikijump.modules.ManageSiteForumSettingsModule.callbacks = {
	saveNesting: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved.";
		w.show();
	},
	activateForum: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Forum has been activated.";
		w.show();
		setTimeout("Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-forum-settings')", 1000);
	}
}
