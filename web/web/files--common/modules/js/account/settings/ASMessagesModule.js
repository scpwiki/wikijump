

Wikijump.modules.ASMessagesModule = {};

Wikijump.modules.ASMessagesModule.listeners = {
	save: function(e){

		var p = OZONE.utils.formToArray("receive-pl-form");

		p.action = "AccountSettingsAction";
		p.event = "saveReceiveMessages";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ASMessagesModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving preferences...";
		w.show();
	}
}

Wikijump.modules.ASMessagesModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Preferences saved."	;
		w.show();

	}

}
