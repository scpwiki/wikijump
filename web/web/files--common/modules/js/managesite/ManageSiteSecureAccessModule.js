

Wikijump.modules.ManageSiteSecureAccessModule = {}

Wikijump.modules.ManageSiteSecureAccessModule.listeners = {
	save: function(e){
		var p = new Object();
		p.action = "ManageSiteAction";
		p.event = "saveSecureAccess";
		p.secureMode = $("sm-ssl-mode-select").value;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteSecureAccessModule.callbacks.save);

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}

}

Wikijump.modules.ManageSiteSecureAccessModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();

		// reload the page!
		window.location.reload();
	}
}
