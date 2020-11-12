

Wikijump.modules.ASNotificationsModule = {};

Wikijump.modules.ASNotificationsModule.listeners = {
	saveReceiveDigest: function(e){
		var receive = $("as-receive-digest").checked;
		var p = new Object();

		p.action = "AccountSettingsAction";
		p.event = "saveReceiveDigest";
		if(receive){
			p.receive = "yes";
		}

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ASNotificationsModule.callbacks.saveReceiveDigest);
	},
	saveReceiveNewsletter: function(e){
		var receive = $("as-receive-newsletter").checked;
		var p = new Object();

		p.action = "AccountSettingsAction";
		p.event = "saveReceiveNewsletter";
		if(receive){
			p.receive = "yes";
		}

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ASNotificationsModule.callbacks.saveReceiveDigest);
	}
}

Wikijump.modules.ASNotificationsModule.callbacks = {
	saveReceiveDigest: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}

}
