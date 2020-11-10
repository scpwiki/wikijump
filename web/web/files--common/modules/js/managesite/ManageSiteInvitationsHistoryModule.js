

Wikijump.modules.ManageSiteInvitationsHistoryModule = {};

Wikijump.modules.ManageSiteInvitationsHistoryModule.vars = {};

Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners = {
	deleteInvitation: function(e, invitationId, email){
		if(confirm("Are you sure you want to delete the invitation for "+email+"?")){
			var p = new Object();
			p.action = "ManageSiteMembershipAction";
			p.event = "deleteEmailInvitation";
			p.invitationId = invitationId;

			OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteInvitationsHistoryModule.callbacks.deleteInvitation);
		}
	},

	resendInvitation: function(e, invitationId, rname, email){
		$("resend-invitations-to").innerHTML = rname+' &lt;'+email+'&gt;';
		$("resend-invitations-form").style.display="block";

		OZONE.visuals.scrollTo($("resend-invitations-form"));

		Wikijump.modules.ManageSiteInvitationsHistoryModule.vars.invitationId = invitationId;
	},

	resendInvitation2: function(e){
		var invitationId = Wikijump.modules.ManageSiteInvitationsHistoryModule.vars.invitationId;
		var p = new Object();
		p.action = "ManageSiteMembershipAction";
		p.event = "resendEmailInvitation";
		p.message = $("resend-invitations-message").value;
		p.invitationId = invitationId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteInvitationsHistoryModule.callbacks.resendInvitation2);
	},

	showAdminOnly: function(e){
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history');

	},
	showAll: function(e){
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history',
		{showAll: true});
	}

}

Wikijump.modules.ManageSiteInvitationsHistoryModule.callbacks = {
	deleteInvitation: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		// reload the module too
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history',
		{showAll: Wikijump.modules.ManageSiteInvitationsHistoryModule.vars.showAll});
	},
	resendInvitation2: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		// reload the module too
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history',
		{showAll: Wikijump.modules.ManageSiteInvitationsHistoryModule.vars.showAll});
	}
}

Wikijump.modules.ManageSiteInvitationsHistoryModule.init = function(){
	// format dates
	OZONE.utils.formatDates("invitations-history-table");
	var showAll = true;
	if($("sm-invhist-showadminonly").style.fontWeight == "bold"){
		showAll = false;
	}
	Wikijump.modules.ManageSiteInvitationsHistoryModule.vars.showAll = showAll;

}

Wikijump.modules.ManageSiteInvitationsHistoryModule.init();
