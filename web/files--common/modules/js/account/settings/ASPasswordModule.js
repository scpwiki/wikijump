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

WIKIDOT.modules.ASPasswordModule = {};

WIKIDOT.modules.ASPasswordModule.listeners = {
	save: function(e){
		var p = OZONE.utils.formToArray("change-password-form");
		p['action'] = "AccountSettingsAction";
		p['event'] = "changePassword";

		var rsa = new RSAKey();
		rsa.setPublic(WIKIDOT.vars.rsakey, "10001");
		
		p['old_password'] = linebrk(hex2b64(rsa.encrypt('__'+p['old_password'])),64);
		p['new_password1'] = linebrk(hex2b64(rsa.encrypt('__'+p['new_password1'])),64);
		p['new_password2'] = linebrk(hex2b64(rsa.encrypt('__'+p['new_password2'])),64);
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASPasswordModule.callbacks.save);
	}	
}

WIKIDOT.modules.ASPasswordModule.callbacks = {
	save: function(r){
		if(r.status == "form_error"){
			var er = $("password-error");
			er.style.display = "block";
			er.innerHTML = r.message;
			return;
		}
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessBox();
			w.content = "Your password has been changed.";
			w.show();
			$("change-password-form").reset();
			setTimeout("WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')", 1000);
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
	}	
}
