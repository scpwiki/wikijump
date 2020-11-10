

Wikijump.modules.CreateAccountModule = {};

Wikijump.modules.CreateAccountModule.vars = {};

Wikijump.modules.CreateAccountModule.listeners = {
	createClick: function(e){
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", null,Wikijump.modules.CreateAccountModule.callbacks.createClick );
	},
	cancel: function(e){
		var p = new Object();
		p.action = "CreateAccountAction";
		p.event = "cancel";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.CreateAccountModule.callbacks.cancel);
	}
}

Wikijump.modules.CreateAccountModule.callbacks = {
	createClick: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();

		// store seed and key
		Wikijump.modules.CreateAccountModule.vars.rsakey = r.key;

	},
	cancel: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		OZONE.dialog.cleanAll();
	}
}
