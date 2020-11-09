

Wikijump.modules.AccountApplicationsModule = {};

Wikijump.modules.AccountApplicationsModule.vars = {
	currentSiteId: null
}

Wikijump.modules.AccountApplicationsModule.listeners = {
	remove: function(e, siteId, siteName){
		Wikijump.modules.AccountApplicationsModule.vars.currentSiteId = siteId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("application-remove-dialog").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, remove'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove', Wikijump.modules.AccountApplicationsModule.listeners.remove2);
		w.show();
	},

	remove2: function(e,siteId0, siteName0){
		if(typeof(siteId0) != 'number'){
			var siteId = Wikijump.modules.AccountApplicationsModule.vars.currentSiteId;
		}else{
			var siteId = siteId0;
		}
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'removeApplication';
		p.site_id = siteId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountApplicationsModule.callbacks.remove );
	}
}

Wikijump.modules.AccountApplicationsModule.callbacks = {
	remove: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The application has been removed.";
		w.show();
		Wikijump.modules.AccountModule.utils.loadModule('am-applications');

	}
}
