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

WIKIDOT.modules.ManageSiteModeratorsModule = {};

WIKIDOT.modules.ManageSiteModeratorsModule.vars = {
	currentUserId: null
}

WIKIDOT.modules.ManageSiteModeratorsModule.listeners = {
	removeModerator: function(event,userId, userName){
		WIKIDOT.modules.ManageSiteModeratorsModule.vars.currentUserId = userId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("remove-moderator-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'yes, remove'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, remove', WIKIDOT.modules.ManageSiteModeratorsModule.listeners.removeModerator2);
		w.show();
	},
	removeModerator2: function(e){
		var userId = WIKIDOT.modules.ManageSiteModeratorsModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeModerator';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteModeratorsModule.callbacks.removeModerator);
	},
	moderatorPermissions: function(e,moderatorId){
		var p = new Object();
		p.moderatorId = moderatorId;
		OZONE.ajax.requestModule("managesite/ManageSiteModeratorPermissionsModule", p, WIKIDOT.modules.ManageSiteModeratorsModule.callbacks.moderatorPermissions);
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
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteModeratorsModule.callbacks.savePermissions);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving permissions...";
		w.show();
	}

}

WIKIDOT.modules.ManageSiteModeratorsModule.callbacks = {
	removeModerator: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "The user has been removed from site moderators.";
		w.show();
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-moderators');
	},
	moderatorPermissions: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		$("mod-permissions-"+r.moderatorId).innerHTML = r.body;
		$("mod-permissions-"+r.moderatorId).style.display = "block";
	
	},
	savePermissions: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Moderator permissions saved.";
		w.show();
	}
}
