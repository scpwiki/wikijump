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

WIKIDOT.modules.CreateAccount0Module = {};

WIKIDOT.modules.CreateAccount0Module.listeners = {
	cancelClick: function(e){
		OZONE.dialog.cleanAll();
	},
	
	nextClick: function(e){
		
		WIKIDOT.modules.CreateAccountModule.vars.formData = OZONE.utils.formToArray("createaccount-form0");
		
		var p = OZONE.utils.formToArray("createaccount-form0");
		//crypt some data please... ;-)
		var rsa = new RSAKey();
		rsa.setPublic(WIKIDOT.modules.CreateAccountModule.vars.rsakey, "10001");
		p['email'] = linebrk(hex2b64(rsa.encrypt('__'+p['email'])),64);
		p['password'] = linebrk(hex2b64(rsa.encrypt('__'+p['password'])),64);
		p['password2'] = linebrk(hex2b64(rsa.encrypt('__'+p['password2'])),64);
		p.action = "CreateAccountAction";
		p.event = "step0";
		OZONE.ajax.requestModule("createaccount/CreateAccount2Module", p, WIKIDOT.modules.CreateAccount0Module.callbacks.nextClick);	
	
	}
}
WIKIDOT.modules.CreateAccount0Module.callbacks = {
	nextClick: function(r){
		if(r.status=="form_errors"){
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";
			
			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}
					
			inner += "</ul>";
			
			$("ca-reg0-errors").style.display = "block";
			$("ca-reg0-errors").innerHTML = inner;
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(r.status == "email_failed"){
			$("ca-reg0-errors").innerHTML = r.message;
			$("ca-reg0-errors").style.display = "block";
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}	

}

WIKIDOT.modules.CreateAccount0Module.init = function(){
	// 	if form data already exists - fill the forms
	if(WIKIDOT.modules.CreateAccountModule.vars.formData != null){
		p = WIKIDOT.modules.CreateAccountModule.vars.formData;
		document.forms.caform['name'].value=p['name'];
		document.forms.caform['password'].value=p['password'];
		document.forms.caform['password2'].value=p['password2'];
		document.forms.caform['email'].value=p['email'];
		document.forms.caform['captcha'].value=p['captcha'];
		if(p['language'] == 'en'){
			$("new-site-lang-en").checked = true;
		}else{
			$("new-site-lang-pl").checked = true;
		}
		document.forms.caform['tos'].checked=true;
	}
}

WIKIDOT.modules.CreateAccount0Module.init();
