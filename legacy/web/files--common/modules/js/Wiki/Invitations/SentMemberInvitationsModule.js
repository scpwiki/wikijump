

Wikijump.modules.SentMemberInvitationsModule = {};

Wikijump.modules.SentMemberInvitationsModule.vars = {};

Wikijump.modules.SentMemberInvitationsModule.listeners = {
	deleteInvitation: function(e, invitationId, email){
		if(confirm("Are you sure you want to delete the invitation for "+email+"?")){
			var p = new Object();
			p.action = "wiki/UserInvitationAction";
			p.event = "deleteEmailInvitation";
			p.invitationId = invitationId;

			OZONE.ajax.requestModule(null, p, Wikijump.modules.SentMemberInvitationsModule.callbacks.deleteInvitation);
		}
	},

	resendInvitation: function(e, invitationId, rname, email){
		$("resend-invitations-to").innerHTML = rname+' &lt;'+email+'&gt;';
		$("resend-invitations-form").style.display="block";

		OZONE.visuals.scrollTo($("resend-invitations-form"));

		Wikijump.modules.SentMemberInvitationsModule.vars.invitationId = invitationId;
	},

	resendInvitation2: function(e){
		var invitationId = Wikijump.modules.SentMemberInvitationsModule.vars.invitationId;
		var p = new Object();
		p.action = "wiki/UserInvitationAction";
		p.event = "resendEmailInvitation";
		p.message = $("resend-invitations-message").value;
		p.invitationId = invitationId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.SentMemberInvitationsModule.callbacks.resendInvitation2);
	},

	sendMore: function(e){
		OZONE.ajax.requestModule("Wiki/Invitations/InviteMembersModule", null, Wikijump.modules.SentMemberInvitationsModule.callbacks.sendMore);
	}

}

Wikijump.modules.SentMemberInvitationsModule.callbacks = {
	deleteInvitation: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		// reload the module too
		Wikijump.modules.SentMemberInvitationsModule.utils.reload();
	},
	resendInvitation2: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		// reload the module too
		Wikijump.modules.SentMemberInvitationsModule.utils.reload();
	},
	sendMore: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("invite-members-module-box").innerHTML = r.body;
	}

}

Wikijump.modules.SentMemberInvitationsModule.utils = {
	reload: function(){
		OZONE.ajax.requestModule("Wiki/Invitations/SentMemberInvitationsModule",null,
			Wikijump.modules.SentMemberInvitationsModule.utils.reloadCallback);
	},

	reloadCallback: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		$("invite-members-module-box").innerHTML = r.body;
	}

}

Wikijump.modules.SentMemberInvitationsModule.init = function(){
	// format dates
	OZONE.utils.formatDates("invitations-history-table");

}

Wikijump.modules.SentMemberInvitationsModule.init();
