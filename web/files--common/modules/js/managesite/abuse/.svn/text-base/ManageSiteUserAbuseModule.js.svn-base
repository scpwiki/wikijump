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

WIKIDOT.modules.ManageSiteUserAbuseModule = {};
WIKIDOT.modules.ManageSiteUserAbuseModule.vars = {};

WIKIDOT.modules.ManageSiteUserAbuseModule.listeners = {
	clear: function(e, userId){
		var p = new Object();
		p.action = "ManageSiteAbuseAction";
		p.event = "clearUserFlags";
		p.userId = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteUserAbuseModule.callbacks.clear);
	},
	removeUser: function(userId, userName){
		WIKIDOT.modules.ManageSiteUserAbuseModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove', WIKIDOT.modules.ManageSiteUserAbuseModule.listeners.removeUser2);
		w.show();
	},
	removeUser2: function(e){
		var userId = WIKIDOT.modules.ManageSiteUserAbuseModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteUserAbuseModule.callbacks.removeUser);
	},
	removeAndBan: function(userId, userName){
		WIKIDOT.modules.ManageSiteUserAbuseModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-ban-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove and ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove and ban', WIKIDOT.modules.ManageSiteUserAbuseModule.listeners.removeAndBan2);
		w.show();
	},
	removeAndBan2: function(e){
		var userId = WIKIDOT.modules.ManageSiteUserAbuseModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeMember';
		p.ban = 'yes';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteUserAbuseModule.callbacks.removeAndBan);
	},
	banUser: function(userId, userName){
		WIKIDOT.modules.ManageSiteUserAbuseModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("ban-user-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, ban', WIKIDOT.modules.ManageSiteUserAbuseModule.listeners.banUser2);
		w.show();
	},
	banUser2: function(e){
		var userId = WIKIDOT.modules.ManageSiteUserAbuseModule.vars.currentUserId;
		var p = new Object();
		p.userId = userId;
		p.action = "ManageSiteBlockAction";
		p.event = "blockUser";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteUserAbuseModule.callbacks.banUser);
	}
	
}

WIKIDOT.modules.ManageSiteUserAbuseModule.callbacks = {
	clear: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Flags cleared";
		w.show();
		
	},
	removeUser: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed.";
		w.show();
		
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-abuse-user');
	},
	removeAndBan: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed and banned.";
		w.show();
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-abuse-user');
	},
	banUser: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The user has been blocked.";
		w.show();
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-abuse-user');
		
	}
}
