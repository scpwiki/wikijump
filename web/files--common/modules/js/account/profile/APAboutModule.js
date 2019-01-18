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

WIKIDOT.modules.APAboutModule = {};

WIKIDOT.modules.APAboutModule.listeners = {
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
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.APAboutModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving profile information...";
		w.show();
	}
}

WIKIDOT.modules.APAboutModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your profile information has been saved.";
		w.show();

	}
}

WIKIDOT.modules.APAboutModule.init = function(){
	YAHOO.util.Event.addListener("about-textarea", "keyup", WIKIDOT.modules.APAboutModule.listeners.aboutChange);	
	WIKIDOT.modules.APAboutModule.listeners.aboutChange();
}

WIKIDOT.modules.APAboutModule.init();
