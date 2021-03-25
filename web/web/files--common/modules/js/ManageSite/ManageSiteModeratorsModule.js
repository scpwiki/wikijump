

Wikijump.modules.ManageSiteModeratorsModule = {};

Wikijump.modules.ManageSiteModeratorsModule.vars = {
	currentUserId: null
}

Wikijump.modules.ManageSiteModeratorsModule.listeners = {
	removeModerator: function(event,userId, userName){
		Wikijump.modules.ManageSiteModeratorsModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-moderator-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove', Wikijump.modules.ManageSiteModeratorsModule.listeners.removeModerator2);
		w.show();
	},
	removeModerator2: function(e){
		var userId = Wikijump.modules.ManageSiteModeratorsModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeModerator';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteModeratorsModule.callbacks.removeModerator);
	},
	moderatorPermissions: function(e,moderatorId){
		var p = new Object();
		p.moderatorId = moderatorId;
		OZONE.ajax.requestModule("ManageSite/ManageSiteModeratorPermissionsModule", p, Wikijump.modules.ManageSiteModeratorsModule.callbacks.moderatorPermissions);
	},

	cancelPermissions: function(e, moderatorId){
		var el = $("mod-permissions-"+moderatorId);
		el.style.display="none";
		el.innerHTML = '';
	},
	savePermissions: function(e,moderatorId){
		var p = OZONE.utils.formToArray("sm-mod-perms-form");
		p.action = 'ManageSiteMembershipAction';
		p.event = 'saveModeratorPermissions';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteModeratorsModule.callbacks.savePermissions);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving permissions...";
		w.show();
	}

}

Wikijump.modules.ManageSiteModeratorsModule.callbacks = {
	removeModerator: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed from site moderators.";
		w.show();
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-moderators');
	},
	moderatorPermissions: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("mod-permissions-"+r.moderatorId).innerHTML = r.body;
		$("mod-permissions-"+r.moderatorId).style.display = "block";

	},
	savePermissions: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Moderator permissions saved.";
		w.show();
	}
}
