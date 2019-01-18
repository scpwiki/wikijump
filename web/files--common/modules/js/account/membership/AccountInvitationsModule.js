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

WIKIDOT.modules.AccountInvitationsModule = {};

WIKIDOT.modules.AccountInvitationsModule.listeners = {
	acceptInvitation: function(e, invitationId){
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'acceptInvitation';
		p.invitation_id = invitationId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountInvitationsModule.callbacks.acceptInvitation);
	},
	
	throwAwayInvitation: function(e, invitationId){
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'throwAwayInvitation';
		p.invitation_id = invitationId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountInvitationsModule.callbacks.throwAwayInvitation);
	}
		
}

WIKIDOT.modules.AccountInvitationsModule.callbacks = {
	acceptInvitation: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = r.message;
			w.show();
			WIKIDOT.modules.AccountModule.utils.loadModule("am-invitations");
		}
	},
	throwAwayInvitation: function(r){
		if(r.status == 'ok'){
			WIKIDOT.modules.AccountModule.utils.loadModule("am-invitations");
		}
	}	
}