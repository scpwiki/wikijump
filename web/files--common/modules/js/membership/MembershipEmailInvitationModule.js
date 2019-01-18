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

WIKIDOT.modules.MembershipEmailInvitationModule = {};

WIKIDOT.modules.MembershipEmailInvitationModule.listeners = {
	accept: function(e, hash){
		
		var p = new Object();
		p.action = 'MembershipApplyAction';
		p.event = 'acceptEmailInvitation';
		p.hash = hash;
		
		OZONE.ajax.requestModule('membership/MembershipEmailInvitationCongratulationModule', p, WIKIDOT.modules.MembershipEmailInvitationModule.callbacks.accept);
		
	}	
	
}

WIKIDOT.modules.MembershipEmailInvitationModule.callbacks = {
	accept: function(r){
		
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		$("membership-email-invitation-box").innerHTML = r.body;
	}	
}