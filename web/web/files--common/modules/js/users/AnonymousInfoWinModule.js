

Wikijump.modules.AnonymousInfoWinModule = {};

Wikijump.modules.AnonymousInfoWinModule.listeners = {
	flagAnonymous: function(e, userString){
		OZONE.ajax.requestModule('report/FlagAnonymousModule', {userString: userString}, Wikijump.modules.AnonymousInfoWinModule.callbacks.flagAnonymous);

	}
}

Wikijump.modules.AnonymousInfoWinModule.callbacks = {
	flagAnonymous: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}
}
