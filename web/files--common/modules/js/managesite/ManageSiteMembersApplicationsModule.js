

Wikijump.modules.ManageSiteMembersApplicationsModule = {};

Wikijump.modules.ManageSiteMembersApplicationsModule.vars = {
	currentUserId: null,
	type: null
}

Wikijump.modules.ManageSiteMembersApplicationsModule.listeners = {
	accept: function(event, userId, userName, type){
		Wikijump.modules.ManageSiteMembersApplicationsModule.vars.currentUserId = userId;
		Wikijump.modules.ManageSiteMembersApplicationsModule.vars.type=type;
		var w = new OZONE.dialogs.Dialog();
		w.title = "Membership application - "+type;
		w.content = $("dialog43").innerHTML.replace(/template\-id\-stub\-/g, 'a-').replace(/%%TYPE%%/g, type).replace(/%%USER_NAME%%/, userName);
		w.buttons = ['cancel', 'send decision'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('send decision', Wikijump.modules.ManageSiteMembersApplicationsModule.listeners.accept2);
		w.show();
		var limiter = new OZONE.forms.lengthLimiter("a-app-area", "a-app-area-left", 200);
	},

	accept2: function(e){
		var userId = Wikijump.modules.ManageSiteMembersApplicationsModule.vars.currentUserId;
		var p = new Object();
		p.action = 'ManageSiteMembershipAction';
		p.event = 'acceptApplication';
		p.user_id = userId;
		p.text = $("a-app-area").value;
		p.type = Wikijump.modules.ManageSiteMembersApplicationsModule.vars.type;

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteMembersApplicationsModule.callbacks.accept);

	}

}

Wikijump.modules.ManageSiteMembersApplicationsModule.callbacks = {
	accept: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The decision has been sent.";
		w.show();
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-ma');

	}

}
