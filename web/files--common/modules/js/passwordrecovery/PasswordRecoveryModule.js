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

WIKIDOT.modules.PasswordRecoveryModule = {};

WIKIDOT.modules.PasswordRecoveryModule.listeners = {
	cancel: function(e){
		var p = new Object();
		p.action = "PasswordRecoveryAction";
		p.event = "cancel";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PasswordRecoveryModule.callbacks.cancel);
	},
	
	next1: function(e){
		var email = $("recovery-email-value").value;
		
		if(email == null || email == ''){
			$("recovery-error").innerHTML = "Email must be provided.";
			$("recovery-error").style.display="block";
			return;
		}
		if(!email.match(/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/)){
			$("recovery-error").innerHTML = "Valid email must be provided.";
			$("recovery-error").style.display="block";
			return;
		}
		
		var rsa = new RSAKey();
		rsa.setPublic(WIKIDOT.vars.rsakey, "10001");
		
		var p = new Object();
		p.email = linebrk(hex2b64(rsa.encrypt('__'+email)),64);
		p.action = "PasswordRecoveryAction";
		p.event = "step1";
		OZONE.ajax.requestModule("passwordrecovery/PasswordRecovery2Module", p, WIKIDOT.modules.PasswordRecoveryModule.callbacks.next1);
	}	,
	
	next2: function(e){
		
		var p = OZONE.utils.formToArray("pr-form");
		p.action = "PasswordRecoveryAction";
		p.event = "step2";
		
		if(p.password != p.password2){
			$("recovery-error").innerHTML = "The passwords are not identical.";
			$("recovery-error").style.display="block";
			return;
		}
		
		//crypt
		var rsa = new RSAKey();
		rsa.setPublic(WIKIDOT.vars.rsakey, "10001");
		p.password = linebrk(hex2b64(rsa.encrypt('__'+p.password)),64);
		p.password2 = linebrk(hex2b64(rsa.encrypt('__'+p.password2)),64);

		OZONE.ajax.requestModule("passwordrecovery/PasswordRecovery3Module", p, WIKIDOT.modules.PasswordRecoveryModule.callbacks.next2);
	}
}

WIKIDOT.modules.PasswordRecoveryModule.callbacks = {
	cancel: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.dialog.cleanAll();
	},
	next1: function(r){
		if(r.status == 'no_email'){
			$("recovery-error").innerHTML = r.message;
			$("recovery-error").style.display="block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		// ok?
		
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
		
	},
	next2: function(r){
		if(r.status == 'form_error'){
			$("recovery-error").innerHTML = r.message;
			$("recovery-error").style.display="block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}
}
