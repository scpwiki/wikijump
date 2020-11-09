
Wikijump.modules.ManageSiteAdminsModule = {};

Wikijump.modules.ManageSiteAdminsModule.vars = {
	currentUserId: null
}

removeAdmin = function(userId, userName){
	Wikijump.modules.ManageSiteAdminsModule.vars.currentUserId = userId;
	var w = new OZONE.dialogs.ConfirmationDialog();
	w.content = $("remove-admin-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
	w.buttons = ['cancel', 'yes, remove'];
	w.addButtonListener('cancel', w.close);
	w.addButtonListener('yes, remove', Wikijump.modules.ManageSiteAdminsModule.listeners.removeAdmin2);
	w.show();
}

Wikijump.modules.ManageSiteAdminsModule.listeners = {
	removeAdmin2: function(e){
		var userId = Wikijump.modules.ManageSiteAdminsModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeAdmin';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteAdminsModule.callbacks.removeAdmin);
	}

}

Wikijump.modules.ManageSiteAdminsModule.callbacks = {
	removeAdmin: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "The user has been removed from site administrators.";
			w.show();
		}else{
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			w.show();
		}
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-admins');
	}
}
