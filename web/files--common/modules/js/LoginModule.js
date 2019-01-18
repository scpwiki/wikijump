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

WIKIDOT.modules.LoginModule = {};

WIKIDOT.modules.LoginModule.listeners = {
	loginClick: function(e){
		parms = OZONE.utils.formToArray("login-form");
		// pre-check:
		if(parms['name'] == '' || parms['password'] == ''){
			message="Both the user name and the password fields should be provided.";
			document.getElementById('loginerror').innerHTML = message;
			return false;
		}
		
		OZONE.ajax.requestModule("login/QuickLoginModule", parms, WIKIDOT.modules.LoginModule.callbacks.loginClick);
	}	
}

WIKIDOT.modules.LoginModule.callbacks = {
	loginClick: function(response){
		if(response.loginValid == false){
			document.getElementById("loginerror").innerHTML=response.errorMessage;
			return false;
		} else {
			// login ok. do sth.
			// request wiki creation module
			window.location.href="/";
		}
	}	
}
