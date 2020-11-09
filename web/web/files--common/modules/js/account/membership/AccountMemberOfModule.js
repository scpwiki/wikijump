

Wikijump.modules.AccountMemberOfModule = {};

Wikijump.modules.AccountMemberOfModule.vars = {
	signOffId : null
}

Wikijump.modules.AccountMemberOfModule.listeners = {
	signOff: function(e, siteInfo){
		Wikijump.modules.AccountMemberOfModule.vars.signOffId = siteInfo[0];
		var siteName = siteInfo[1];
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("signoff-window").innerHTML.replace(/%%SITE_NAME%%/, siteName);
		w.buttons = ['cancel', 'yes, sign me off'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, sign me off', Wikijump.modules.AccountMemberOfModule.listeners.signOff2);
		w.show();
	},

	signOff2: function(e){
		var siteId = Wikijump.modules.AccountMemberOfModule.vars.signOffId;
		var p = new Object();
		p.site_id = siteId;
		p.action = 'AccountMembershipAction';
		p.event = 'signOff';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountMemberOfModule.callbacks.signOff);
	}

}

Wikijump.modules.AccountMemberOfModule.callbacks = {
	signOff: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "You have successfully signed off from this site";
			w.show();
			Wikijump.modules.AccountModule.utils.loadModule("am-memberof");
		}else{
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			w.show();
		}

	}
}
