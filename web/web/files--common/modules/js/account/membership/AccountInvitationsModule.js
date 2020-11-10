

Wikijump.modules.AccountInvitationsModule = {};

Wikijump.modules.AccountInvitationsModule.listeners = {
	acceptInvitation: function(e, invitationId){
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'acceptInvitation';
		p.invitation_id = invitationId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountInvitationsModule.callbacks.acceptInvitation);
	},

	throwAwayInvitation: function(e, invitationId){
		var p = new Object();
		p.action = 'AccountMembershipAction';
		p.event = 'throwAwayInvitation';
		p.invitation_id = invitationId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountInvitationsModule.callbacks.throwAwayInvitation);
	}

}

Wikijump.modules.AccountInvitationsModule.callbacks = {
	acceptInvitation: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = r.message;
			w.show();
			Wikijump.modules.AccountModule.utils.loadModule("am-invitations");
		}
	},
	throwAwayInvitation: function(r){
		if(r.status == 'ok'){
			Wikijump.modules.AccountModule.utils.loadModule("am-invitations");
		}
	}
}
