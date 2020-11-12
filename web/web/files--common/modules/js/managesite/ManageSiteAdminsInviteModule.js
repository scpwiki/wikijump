

Wikijump.modules.ManagerSiteAdminsInviteModule = {};

Wikijump.modules.ManagerSiteAdminsInviteModule.vars = {};

Wikijump.modules.ManagerSiteAdminsInviteModule.listeners = {

	searchClick: function(e){
		var query = document.getElementById("sm-ad-search-f").value;
		var parms = new Array();
		parms['query'] = query;
		OZONE.ajax.requestModule("users/UserSearchModule", parms, Wikijump.modules.ManagerSiteAdminsInviteModule.callbacks.searchClick);
	},

	inviteAdmin: function(e){
		// display a nice dialog box...
		var userId = this.id.replace("invite-admin-b-", '');
		var userName = Wikijump.modules.ManagerSiteAdminsInviteModule.vars.userNames[userId];
		Wikijump.modules.ManagerSiteAdminsInviteModule.vars.userId = userId;
		var shader = OZONE.dialog.factory.shader();
		var container = OZONE.dialog.factory.boxcontainer();

		shader.show();
		container.setContent(document.getElementById("sm-tmp-not").innerHTML.replace(/id="/g, 'id="s').replace(/%%USERNAME%%/g,userName));
		container.showContent();
		YAHOO.util.Event.addListener("s-cancel", "click", Wikijump.modules.ManagerSiteAdminsInviteModule.listeners.cancelInvitation);
		YAHOO.util.Event.addListener("s-send", "click", Wikijump.modules.ManagerSiteAdminsInviteModule.listeners.inviteAdmin2);

	},

	cancelInvitation: function(){
		var container = OZONE.dialog.factory.boxcontainer();
		container.hideContent();
		OZONE.dialog.cleanAll();
	},

	inviteAdmin2: function(e){
		var parms = new Array();
		parms['action'] = 'ManageSiteAction';
		parms['event'] = "inviteAdmin";
		parms['user_id'] = Wikijump.modules.ManagerSiteAdminsInviteModule.vars.userId;
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManagerSiteAdminsInviteModule.callbacks.inviteAdmin);
	}

}

Wikijump.modules.ManagerSiteAdminsInviteModule.callbacks = {

	searchClick:	 function(response){
		OZONE.utils.setInnerHTMLContent("sm-ad-search-r", response.body);
		Wikijump.modules.ManagerSiteAdminsInviteModule.vars.searchCount = response.count;

		// now modify the fields to include "add as admin"
		var userIds = response.userIds;
		var buttonsIds = new Array();
		for(var i=0;i<userIds.length; i++){
			var	divid = "found-user-"+userIds[i];
			el = document.getElementById(divid);
			el.innerHTML += '(<a href="javascript:;" id="invite-admin-b-'+userIds[i]+'">invite</a>)';
			YAHOO.util.Event.addListener("invite-admin-b-"+userIds[i], "click", Wikijump.modules.ManagerSiteAdminsInviteModule.listeners.inviteAdmin);
		}

		Wikijump.modules.ManagerSiteAdminsInviteModule.vars.userNames = response.userNames;

	},

	inviteAdmin: function(response){
		if(response.result == 'invited'){
			var divid = "found-user-"+response.userId;
			alert("invited "+divid);
			var eff = new fx.Opacity(divid, {duration: 200});
			eff.custom(1,0);
			OZONE.dialog.cleanAll();
			setTimeout('document.getElementById("'+divid+'").style.display="none"',300);
		}
	}

}

Wikijump.modules.ManagerSiteAdminsInviteModule.init = function(){
	YAHOO.util.Event.addListener("sm-ad-search-b", "click",Wikijump.modules.ManagerSiteAdminsInviteModule.listeners.searchClick);
}

Wikijump.modules.ManagerSiteAdminsInviteModule.init();
