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

WIKIDOT.modules.SentMemberInvitationsModule = {};

WIKIDOT.modules.SentMemberInvitationsModule.vars = {};

WIKIDOT.modules.SentMemberInvitationsModule.listeners = {
	deleteInvitation: function(e, invitationId, email){
		if(confirm("Are you sure you want to delete the invitation for "+email+"?")){
			var p = new Object();
			p.action = "wiki/UserInvitationAction";
			p.event = "deleteEmailInvitation";			
			p.invitationId = invitationId;
			
			OZONE.ajax.requestModule(null, p, WIKIDOT.modules.SentMemberInvitationsModule.callbacks.deleteInvitation);
		}
	},
	
	resendInvitation: function(e, invitationId, rname, email){
		$("resend-invitations-to").innerHTML = rname+' &lt;'+email+'&gt;';
		$("resend-invitations-form").style.display="block";
		
		OZONE.visuals.scrollTo($("resend-invitations-form"));
	
		WIKIDOT.modules.SentMemberInvitationsModule.vars.invitationId = invitationId;
	},
	
	resendInvitation2: function(e){
		var invitationId = WIKIDOT.modules.SentMemberInvitationsModule.vars.invitationId;
		var p = new Object();
		p.action = "wiki/UserInvitationAction";
		p.event = "resendEmailInvitation";		
		p.message = $("resend-invitations-message").value;
		p.invitationId = invitationId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.SentMemberInvitationsModule.callbacks.resendInvitation2);	
	},
	
	sendMore: function(e){
		OZONE.ajax.requestModule("wiki/invitations/InviteMembersModule", null, WIKIDOT.modules.SentMemberInvitationsModule.callbacks.sendMore);
	}
	
}

WIKIDOT.modules.SentMemberInvitationsModule.callbacks = {
	deleteInvitation: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		// reload the module too
		WIKIDOT.modules.SentMemberInvitationsModule.utils.reload();
	},
	resendInvitation2: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		// reload the module too
		WIKIDOT.modules.SentMemberInvitationsModule.utils.reload();
	},
	sendMore: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("invite-members-module-box").innerHTML = r.body;
	}
	
}

WIKIDOT.modules.SentMemberInvitationsModule.utils = {
	reload: function(){
		OZONE.ajax.requestModule("wiki/invitations/SentMemberInvitationsModule",null,
			WIKIDOT.modules.SentMemberInvitationsModule.utils.reloadCallback);
	},
	
	reloadCallback: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		$("invite-members-module-box").innerHTML = r.body;
	}
	
}

WIKIDOT.modules.SentMemberInvitationsModule.init = function(){
	// format dates
	OZONE.utils.formatDates("invitations-history-table");
	
}

WIKIDOT.modules.SentMemberInvitationsModule.init();