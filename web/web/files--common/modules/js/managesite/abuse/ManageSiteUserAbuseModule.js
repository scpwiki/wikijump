

Wikijump.modules.ManageSiteUserAbuseModule = {};
Wikijump.modules.ManageSiteUserAbuseModule.vars = {};

Wikijump.modules.ManageSiteUserAbuseModule.listeners = {
	clear: function(e, userId){
		var p = new Object();
		p.action = "ManageSiteAbuseAction";
		p.event = "clearUserFlags";
		p.userId = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteUserAbuseModule.callbacks.clear);
	},
	removeUser: function(userId, userName){
		Wikijump.modules.ManageSiteUserAbuseModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove', Wikijump.modules.ManageSiteUserAbuseModule.listeners.removeUser2);
		w.show();
	},
	removeUser2: function(e){
		var userId = Wikijump.modules.ManageSiteUserAbuseModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteUserAbuseModule.callbacks.removeUser);
	},
	removeAndBan: function(userId, userName){
		Wikijump.modules.ManageSiteUserAbuseModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-ban-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove and ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove and ban', Wikijump.modules.ManageSiteUserAbuseModule.listeners.removeAndBan2);
		w.show();
	},
	removeAndBan2: function(e){
		var userId = Wikijump.modules.ManageSiteUserAbuseModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.ban = 'yes';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteUserAbuseModule.callbacks.removeAndBan);
	},
	banUser: function(userId, userName){
		Wikijump.modules.ManageSiteUserAbuseModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("ban-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, ban', Wikijump.modules.ManageSiteUserAbuseModule.listeners.banUser2);
		w.show();
	},
	banUser2: function(e){
		var userId = Wikijump.modules.ManageSiteUserAbuseModule.vars.currentUserId;
		var p = new Object();
		p.userId = userId;
		p.action = "ManageSiteBlockAction";
		p.event = "blockUser";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteUserAbuseModule.callbacks.banUser);
	}

}

Wikijump.modules.ManageSiteUserAbuseModule.callbacks = {
	clear: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Flags cleared";
		w.show();

	},
	removeUser: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed.";
		w.show();

		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-abuse-user');
	},
	removeAndBan: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed and banned.";
		w.show();
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-abuse-user');
	},
	banUser: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The user has been blocked.";
		w.show();
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-abuse-user');

	}
}
