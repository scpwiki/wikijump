

Wikijump.modules.ASLanguageModule = {};

Wikijump.modules.ASLanguageModule.listeners = {
	save: function(e){
		var lang = $("as-language-select").value;
		var p = new Object();

		p.action = "AccountSettingsAction";
		p.event = "saveLanguage";

		p.language = lang;

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ASLanguageModule.callbacks.save);
	}
}

Wikijump.modules.ASLanguageModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
		var lang = r.language;
		var url;
		url = HTTP_SCHEMA+"://"+URL_HOST+"/account:you";
		setTimeout("window.location.href='"+url+"'", 1500);
	}

}
