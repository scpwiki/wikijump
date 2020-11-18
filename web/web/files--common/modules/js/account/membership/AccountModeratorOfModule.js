

Wikijump.modules.AccountModeratorOfModule = {};

Wikijump.modules.AccountModeratorOfModule.vars = {
	currentSiteId: null
}

Wikijump.modules.AccountModeratorOfModule.listeners = {
	resign: function(e, siteId, siteName){
		Wikijump.modules.AccountModeratorOfModule.vars.currentSiteId = siteId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("moderator-resign-dialog").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, resign'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, resign', Wikijump.modules.AccountModeratorOfModule.listeners.resign2);
		w.show();
	},

	resign2: function(e){
		var siteId = Wikijump.modules.AccountModeratorOfModule.vars.currentSiteId;
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'moderatorResign';
		p.site_id = siteId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountModeratorOfModule.callbacks.resign );
	}
}

Wikijump.modules.AccountModeratorOfModule.callbacks = {
	resign: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "You are no longer a moderator of this site.";
			w.show();
			Wikijump.modules.AccountModule.utils.loadModule('am-moderatorof');
		} else {
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			w.show();

		}
	}
}
