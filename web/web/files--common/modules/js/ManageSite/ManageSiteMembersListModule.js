

Wikijump.modules.ManageSiteMembersListModule = {};

Wikijump.modules.ManageSiteMembersListModule.vars = {
	currentUserId: null
}

removeUser = function(userId, userName){
	Wikijump.modules.ManageSiteMembersListModule.vars.currentUserId = userId;
	var w = new OZONE.dialogs.ConfirmationDialog();
	w.content = $("remove-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
	w.buttons = ['cancel', 'yes, remove'];
	w.addButtonListener('cancel', w.close);
	w.addButtonListener('yes, remove', Wikijump.modules.ManageSiteMembersListModule.listeners.removeUser2);
	w.show();
}

toModerators = function(userId){
	var p = new Object();
	p.action = 'ManageSiteMembershipAction';
	p.event = 'toModerators';
	p.user_id = userId;
	OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteMembersListModule.callbacks.toModerators);
}

toAdmins = function(userId){
	var p = new Object();
	p.action = 'ManageSiteMembershipAction';
	p.event = 'toAdmins';
	p.user_id = userId;
	OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteMembersListModule.callbacks.toAdmins);
}

Wikijump.modules.ManageSiteMembersListModule.listeners = {
	removeUser2: function(e){
		var userId = Wikijump.modules.ManageSiteMembersListModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteMembersListModule.callbacks.removeUser);
	},

	removeAndBan: function(userId, userName){
		Wikijump.modules.ManageSiteMembersListModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-ban-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove and ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove and ban', Wikijump.modules.ManageSiteMembersListModule.listeners.removeAndBan2);
		w.show();
	},
	removeAndBan2: function(e){
		var userId = Wikijump.modules.ManageSiteMembersListModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.ban = 'yes';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteMembersListModule.callbacks.removeAndBan);
	}

}

Wikijump.modules.ManageSiteMembersListModule.callbacks = {
	removeUser: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed.";
		w.show();

		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-members-list');
	},
	toModerators: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user <strong>"+r.userName+"</strong> has been added to moderators.<br/>" +
				"Now please go to the list of moderators and set new permissions.";
		w.show();
	},
	toAdmins: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user <strong>"+r.userName+"</strong> has been added to site administrators.";
		w.show();

	},
	removeAndBan: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed and banned.";
		w.show();

		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-members-list');
	}
}
