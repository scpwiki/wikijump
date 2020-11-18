

Wikijump.modules.AccountAdminOfModule = {};

Wikijump.modules.AccountAdminOfModule.vars = {
	currentSiteId: null
}

Wikijump.modules.AccountAdminOfModule.listeners = {
	resign: function(e, siteId, siteName){
		Wikijump.modules.AccountAdminOfModule.vars.currentSiteId = siteId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("admin-resign-dialog").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, resign'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, resign', Wikijump.modules.AccountAdminOfModule.listeners.resign2);
		w.show();
	},

	resign2: function(e){
		var siteId = Wikijump.modules.AccountAdminOfModule.vars.currentSiteId;
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'adminResign';
		p.site_id = siteId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountAdminOfModule.callbacks.resign );
	}
}

Wikijump.modules.AccountAdminOfModule.callbacks = {
	resign: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "You are no longer an admin of this site.";
			w.show();
			Wikijump.modules.AccountModule.utils.loadModule('am-adminof');
		} else {
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			w.show();

		}
	}
}
