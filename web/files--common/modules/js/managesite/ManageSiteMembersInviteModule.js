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

WIKIDOT.modules.ManagerSiteMembersInviteModule = {};

WIKIDOT.modules.ManagerSiteMembersInviteModule.vars = {};

WIKIDOT.modules.ManagerSiteMembersInviteModule.listeners = {

	searchClick: function(e){
		var query = $("sm-mi-search-f").value;
		YAHOO.util.Event.preventDefault(e);
		if(query.length<2){
			$("sm-mi-search-r").innerHTML = "please type at least 2 characters..."
			return;
		}
		var parms = new Object();
		parms['query'] = query;
		OZONE.ajax.requestModule("users/UserSearchModule", parms, WIKIDOT.modules.ManagerSiteMembersInviteModule.callbacks.searchClick);
		
	},
	
	inviteMember: function(e){
		// display a nice dialog box...
		var userId = this.id.replace("invite-member-b-", '');
		var userName = WIKIDOT.modules.ManagerSiteMembersInviteModule.vars.userNames[userId];
		WIKIDOT.modules.ManagerSiteMembersInviteModule.vars.userId = userId;
		
		var w = new OZONE.dialogs.Dialog();
		var reg = new RegExp('template-id-stub','g');
		w.content = $("sm-tmp-not").innerHTML.replace(reg, 's').replace(/%%USERNAME%%/g,userName);
		w.show()
		
		YAHOO.util.Event.addListener("s-cancel", "click", WIKIDOT.modules.ManagerSiteMembersInviteModule.listeners.cancelInvitation);
		YAHOO.util.Event.addListener("s-send", "click", WIKIDOT.modules.ManagerSiteMembersInviteModule.listeners.inviteMember2);
		var limiter = new OZONE.forms.lengthLimiter("s-text", "s-charleft", 200);
		
	},
	
	cancelInvitation: function(){
		var container = OZONE.dialog.factory.boxcontainer();
		container.hideContent();
		OZONE.dialog.cleanAll();
	},
	
	inviteMember2: function(e){
		var parms = new Array();
		parms['action'] = 'ManageSiteMembershipAction';
		parms['event'] = "inviteMember";
		parms['user_id'] = WIKIDOT.modules.ManagerSiteMembersInviteModule.vars.userId;
		parms['text'] = $("s-text").value;
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManagerSiteMembersInviteModule.callbacks.inviteMember);
	}
	
}

WIKIDOT.modules.ManagerSiteMembersInviteModule.callbacks = {

	searchClick:	 function(response){
		if(!WIKIDOT.utils.handleError(response)) {return;}
		
		OZONE.utils.setInnerHTMLContent("sm-mi-search-r", response.body);
		WIKIDOT.modules.ManagerSiteMembersInviteModule.vars.searchCount = response.count;
		
		// now modify the fields to include "add as member"
	
		var userIds = response.userIds;
		var buttonsIds = new Array();
		for(var i=0;i<userIds.length; i++){
			var	divid = "found-user-"+userIds[i];
			el = $(divid);
			el.innerHTML += '(<a href="javascript:;" id="invite-member-b-'+userIds[i]+'">invite</a>)';
			YAHOO.util.Event.addListener("invite-member-b-"+userIds[i], "click", WIKIDOT.modules.ManagerSiteMembersInviteModule.listeners.inviteMember);
		}
		
		WIKIDOT.modules.ManagerSiteMembersInviteModule.vars.userNames = response.userNames;

	},
	
	inviteMember: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The invitation has been sent";
		w.show();

	}

}

WIKIDOT.modules.ManagerSiteMembersInviteModule.init = function(){
	YAHOO.util.Event.addListener("sm-mi-search-b", "click",WIKIDOT.modules.ManagerSiteMembersInviteModule.listeners.searchClick);
	YAHOO.util.Event.addListener("sm-search-user", "submit",WIKIDOT.modules.ManagerSiteMembersInviteModule.listeners.searchClick);
	
}

WIKIDOT.modules.ManagerSiteMembersInviteModule.init();
