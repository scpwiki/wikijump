

Wikijump.modules.ManagerSiteMembersModule = {};

Wikijump.modules.ManagerSiteMembersModule.listeners = {
	save: function(e){
		var parms = OZONE.utils.formToArray("sm-mem-form");
		parms['action'] = "ManageSiteMembershipAction";
		parms['event'] = "saveMemberPolicy";
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManagerSiteMembersModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();

	},
	cancel: function(e){
		OZONE.ajax.requestModule("managesite/ManageSiteModule", null, Wikijump.modules.ManagerSiteMembersModule.callbacks.cancel)
	}
}

Wikijump.modules.ManagerSiteMembersModule.callbacks = {
	save: function(){
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	},
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	}

}

Wikijump.modules.ManagerSiteMembersModule.init = function(){
	YAHOO.util.Event.addListener("sm-members-cancel", "click", Wikijump.modules.ManagerSiteMembersModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-members-save", "click", Wikijump.modules.ManagerSiteMembersModule.listeners.save);

}

Wikijump.modules.ManagerSiteMembersModule.init();
