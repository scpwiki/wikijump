

Wikijump.modules.LoginModule = {};

Wikijump.modules.LoginModule.listeners = {
	loginClick: function(e){
		parms = OZONE.utils.formToArray("login-form");
		// pre-check:
		if(parms['name'] == '' || parms['password'] == ''){
			message="Both the user name and the password fields should be provided.";
			document.getElementById('loginerror').innerHTML = message;
			return false;
		}

		OZONE.ajax.requestModule("login/QuickLoginModule", parms, Wikijump.modules.LoginModule.callbacks.loginClick);
	}
}

Wikijump.modules.LoginModule.callbacks = {
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
