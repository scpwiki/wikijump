/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.modules.ASLanguageModule = {};

WIKIDOT.modules.ASLanguageModule.listeners = {
	save: function(e){
		var lang = $("as-language-select").value;
		var p = new Object();
		
		p.action = "AccountSettingsAction";
		p.event = "saveLanguage";
		
		p.language = lang;
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASLanguageModule.callbacks.save);
	}	
}

WIKIDOT.modules.ASLanguageModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
		var lang = r.language;
		var url;
		url = "http://"+URL_HOST+"/account:you";
		setTimeout("window.location.href='"+url+"'", 1500);
	}
		
}
