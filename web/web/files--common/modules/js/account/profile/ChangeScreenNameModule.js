

Wikijump.modules.ChangeScreenNameModule = {};

Wikijump.modules.ChangeScreenNameModule.listeners = {
	save: function(e){
		var p = {};
		p['action'] = "AccountProfileAction";
		p['event'] = "changeScreenName";
		p.screenName = $("ap-screen-name-input").value;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ChangeScreenNameModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Changing the screen name...";
		w.show();
		YAHOO.util.Event.stopEvent(e);
	}
}

Wikijump.modules.ChangeScreenNameModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your screen name has been changed!";
		w.show();

		OZONE.ajax.requestModule('account/profile/ChangeScreenNameModule', null, Wikijump.modules.AccountModule.callbacks.menuClick);

	}
}

