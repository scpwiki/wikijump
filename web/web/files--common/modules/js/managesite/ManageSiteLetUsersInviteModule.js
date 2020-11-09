

Wikijump.modules.ManageSiteLetUsersInviteModule = {}

Wikijump.modules.ManageSiteLetUsersInviteModule.vars = {
	randoms: new Object(),
	services: null
}

Wikijump.modules.ManageSiteLetUsersInviteModule.listeners = {

	save: function(e){
		var p = new Object();
		p.action = "ManageSiteMembershipAction";
		p.event = "letUsersInviteSave";
		p.enableLetUsersInvite = $("sm-allow-users-invite").checked;

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteLetUsersInviteModule.callbacks.save);

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}

}

Wikijump.modules.ManageSiteLetUsersInviteModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
}
Wikijump.modules.ManageSiteLetUsersInviteModule.init = function(){

}

Wikijump.modules.ManageSiteLetUsersInviteModule.init();
