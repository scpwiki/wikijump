

Wikijump.modules.ASInvitationsModule = {};

Wikijump.modules.ASInvitationsModule.listeners = {
	save: function(e){
		var val = $("receive-invitations-ch").checked;
		var p = new Object();
		if(val){
			p.receive = true;
		}
		p.action = "AccountSettingsAction";
		p.event = "saveReceiveInvitations";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ASInvitationsModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving preferences...";
		w.show();
	}
}

Wikijump.modules.ASInvitationsModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Preferences saved."	;
		w.show();

	}

}
