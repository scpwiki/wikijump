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

WIKIDOT.modules.ManageSiteAdminsModule = {};

WIKIDOT.modules.ManageSiteAdminsModule.vars = {
	currentUserId: null
}

removeAdmin = function(userId, userName){
	WIKIDOT.modules.ManageSiteAdminsModule.vars.currentUserId = userId;
	var w = new OZONE.dialogs.ConfirmationDialog();
	w.content = $("remove-admin-dialog").innerHTML.replace(/%%USER_NAME%%/, userName);
	w.buttons = ['cancel', 'yes, remove'];
	w.addButtonListener('cancel', w.close);
	w.addButtonListener('yes, remove', WIKIDOT.modules.ManageSiteAdminsModule.listeners.removeAdmin2);
	w.show();
}

WIKIDOT.modules.ManageSiteAdminsModule.listeners = {
	removeAdmin2: function(e){
		var userId = WIKIDOT.modules.ManageSiteAdminsModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'removeAdmin';
		p.user_id = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteAdminsModule.callbacks.removeAdmin);
	}

}

WIKIDOT.modules.ManageSiteAdminsModule.callbacks = {
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
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-admins');
	}
}
