

Wikijump.modules.MembershipEmailInvitationModule = {};

Wikijump.modules.MembershipEmailInvitationModule.listeners = {
	accept: function(e, hash){

		var p = new Object();
		p.action = 'MembershipApplyAction';
		p.event = 'acceptEmailInvitation';
		p.hash = hash;

		OZONE.ajax.requestModule('Membership/MembershipEmailInvitationCongratulationModule', p, Wikijump.modules.MembershipEmailInvitationModule.callbacks.accept);

	}

}

Wikijump.modules.MembershipEmailInvitationModule.callbacks = {
	accept: function(r){

		if(!Wikijump.utils.handleError(r)) {return;}

		$("membership-email-invitation-box").innerHTML = r.body;
	}
}
