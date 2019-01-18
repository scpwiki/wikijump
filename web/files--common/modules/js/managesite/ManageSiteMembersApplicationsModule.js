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

WIKIDOT.modules.ManageSiteMembersApplicationsModule = {};

WIKIDOT.modules.ManageSiteMembersApplicationsModule.vars = {
	currentUserId: null,
	type: null
}

WIKIDOT.modules.ManageSiteMembersApplicationsModule.listeners = {
	accept: function(event, userId, userName, type){
		WIKIDOT.modules.ManageSiteMembersApplicationsModule.vars.currentUserId = userId;
		WIKIDOT.modules.ManageSiteMembersApplicationsModule.vars.type=type;
		var w = new OZONE.dialogs.Dialog();
		w.title = "Membership application - "+type;
		w.content = $("dialog43").innerHTML.replace(/template\-id\-stub\-/g, 'a-').replace(/%%TYPE%%/g, type).replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'send decision'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('send decision', WIKIDOT.modules.ManageSiteMembersApplicationsModule.listeners.accept2);
		w.show();
		var limiter = new OZONE.forms.lengthLimiter("a-app-area", "a-app-area-left", 200);
	},
	
	accept2: function(e){
		var userId = WIKIDOT.modules.ManageSiteMembersApplicationsModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'acceptApplication';
		p.user_id = userId;
		p.text = $("a-app-area").value;
		p.type = WIKIDOT.modules.ManageSiteMembersApplicationsModule.vars.type;
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteMembersApplicationsModule.callbacks.accept);
		
	}
	
}

WIKIDOT.modules.ManageSiteMembersApplicationsModule.callbacks = {
	accept: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The decision has been sent.";
		w.show();
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-ma');
		
	}

}
