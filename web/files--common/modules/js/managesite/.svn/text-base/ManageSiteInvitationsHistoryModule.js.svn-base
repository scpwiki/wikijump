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

WIKIDOT.modules.ManageSiteInvitationsHistoryModule = {};

WIKIDOT.modules.ManageSiteInvitationsHistoryModule.vars = {};

WIKIDOT.modules.ManageSiteInvitationsHistoryModule.listeners = {
	deleteInvitation: function(e, invitationId, email){
		if(confirm("Are you sure you want to delete the invitation for "+email+"?")){
			var p = new Object();
			p.action = "ManageSiteMembershipAction";
			p.event = "deleteEmailInvitation";			
			p.invitationId = invitationId;
			
			OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteInvitationsHistoryModule.callbacks.deleteInvitation);
		}
	},
	
	resendInvitation: function(e, invitationId, rname, email){
		$("resend-invitations-to").innerHTML = rname+' &lt;'+email+'&gt;';
		$("resend-invitations-form").style.display="block";
		
		OZONE.visuals.scrollTo($("resend-invitations-form"));
	
		WIKIDOT.modules.ManageSiteInvitationsHistoryModule.vars.invitationId = invitationId;
	},
	
	resendInvitation2: function(e){
		var invitationId = WIKIDOT.modules.ManageSiteInvitationsHistoryModule.vars.invitationId;
		var p = new Object();
		p.action = "ManageSiteMembershipAction";
		p.event = "resendEmailInvitation";		
		p.message = $("resend-invitations-message").value;
		p.invitationId = invitationId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteInvitationsHistoryModule.callbacks.resendInvitation2);	
	},
	
	showAdminOnly: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history');
		
	},
	showAll: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history',
		{showAll: true});
	}
	
}

WIKIDOT.modules.ManageSiteInvitationsHistoryModule.callbacks = {
	deleteInvitation: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		// reload the module too
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history',
		{showAll: WIKIDOT.modules.ManageSiteInvitationsHistoryModule.vars.showAll});
	},
	resendInvitation2: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		// reload the module too
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history',
		{showAll: WIKIDOT.modules.ManageSiteInvitationsHistoryModule.vars.showAll});
	}
}

WIKIDOT.modules.ManageSiteInvitationsHistoryModule.init = function(){
	// format dates
	OZONE.utils.formatDates("invitations-history-table");
	var showAll = true;
	if($("sm-invhist-showadminonly").style.fontWeight == "bold"){
		showAll = false;
	}
	WIKIDOT.modules.ManageSiteInvitationsHistoryModule.vars.showAll = showAll;
	
}

WIKIDOT.modules.ManageSiteInvitationsHistoryModule.init();