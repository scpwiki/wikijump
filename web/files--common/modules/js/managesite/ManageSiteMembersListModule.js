/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.modules.ManageSiteMembersListModule = {};

WIKIDOT.modules.ManageSiteMembersListModule.vars = {
	currentUserId: null
}

removeUser = function(userId, userName){
	WIKIDOT.modules.ManageSiteMembersListModule.vars.currentUserId = userId;
	var w = new OZONE.dialogs.ConfirmationDialog();
	w.content = $("remove-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
	w.buttons = ['cancel', 'yes, remove'];
	w.addButtonListener('cancel', w.close);
	w.addButtonListener('yes, remove', WIKIDOT.modules.ManageSiteMembersListModule.listeners.removeUser2);
	w.show();
}

toModerators = function(userId){
	var p = new Object();
	p.action = 'ManageSiteMembershipAction';
	p.event = 'toModerators';
	p.user_id = userId;
	OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteMembersListModule.callbacks.toModerators);
}

toAdmins = function(userId){
	var p = new Object();
	p.action = 'ManageSiteMembershipAction';
	p.event = 'toAdmins';
	p.user_id = userId;
	OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteMembersListModule.callbacks.toAdmins);
}

WIKIDOT.modules.ManageSiteMembersListModule.listeners = {
	removeUser2: function(e){
		var userId = WIKIDOT.modules.ManageSiteMembersListModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteMembersListModule.callbacks.removeUser);
	},
	
	removeAndBan: function(userId, userName){
		WIKIDOT.modules.ManageSiteMembersListModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-ban-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove and ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove and ban', WIKIDOT.modules.ManageSiteMembersListModule.listeners.removeAndBan2);
		w.show();
	},
	removeAndBan2: function(e){
		var userId = WIKIDOT.modules.ManageSiteMembersListModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.ban = 'yes';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteMembersListModule.callbacks.removeAndBan);
	}

}

WIKIDOT.modules.ManageSiteMembersListModule.callbacks = {
	removeUser: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed.";
		w.show();
		
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-members-list');
	},
	toModerators: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user <strong>"+r.userName+"</strong> has been added to moderators.<br/>" +
				"Now please go to the list of moderators and set new permissions.";
		w.show();
	},
	toAdmins: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user <strong>"+r.userName+"</strong> has been added to site administrators.";
		w.show();
		
	},
	removeAndBan: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed and banned.";
		w.show();
		
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-members-list');
	}
}
