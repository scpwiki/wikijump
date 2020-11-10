

Wikijump.modules.APAboutModule = {};

Wikijump.modules.APAboutModule.listeners = {
	aboutChange: function(e){
		// get number of characters...
		var chars = $("about-textarea").value.replace(/\r\n/, "\n").length;
		$("chleft").innerHTML = 200 - chars;
		if(chars>200){
			var scrollTop = $("about-textarea").scrollTop;
			$("about-textarea").value = $("about-textarea").value.substr(0,200);
			$("about-textarea").scrollTop = scrollTop;
			var chars = $("about-textarea").value.replace(/\r\n/, "\n").length;
			$("chleft").innerHTML = 200 - chars;
		}
	},
	save: function(e){
		var p = OZONE.utils.formToArray("about-form");
		p['action'] = "AccountProfileAction";
		p['event'] = "saveAbout";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.APAboutModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving profile information...";
		w.show();
	}
}

Wikijump.modules.APAboutModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your profile information has been saved.";
		w.show();

	}
}

Wikijump.modules.APAboutModule.init = function(){
	YAHOO.util.Event.addListener("about-textarea", "keyup", Wikijump.modules.APAboutModule.listeners.aboutChange);
	Wikijump.modules.APAboutModule.listeners.aboutChange();
}

Wikijump.modules.APAboutModule.init();
