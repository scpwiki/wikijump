

Wikijump.modules.CreateAccountStep1Module = {};

Wikijump.modules.CreateAccountStep1Module.vars = {};

Wikijump.modules.CreateAccountStep1Module.listeners = {
	cancel: function(e){
		window.location.href = HTTP_SCHEMA+"://"+window.location.hostname;
	},

	nextClick: function(e){

		Wikijump.modules.CreateAccountStep1Module.vars.formData = OZONE.utils.formToArray("createaccount-form0");

		var p = OZONE.utils.formToArray("createaccount-form0");

		p.action = "CreateAccountStep2Action";
		p.event = "step0";
		OZONE.ajax.requestModule("CreateAccount/CreateAccountStep2Module", p, Wikijump.modules.CreateAccountStep1Module.callbacks.nextClick);

	}
}
Wikijump.modules.CreateAccountStep1Module.callbacks = {
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
			return;
		}
		if(r.status == "email_failed"){
			$("ca-reg0-errors").innerHTML = r.message;
			$("ca-reg0-errors").style.display = "block";
			return;
		}
		window.location.href='/auth:newaccount2';
	}

}

Wikijump.modules.CreateAccountStep1Module.init = function(){
	// 	if form data already exists - fill the forms
	if(Wikijump.modules.CreateAccountStep1Module.vars.formData != null){
		p = Wikijump.modules.CreateAccountStep1Module.vars.formData;
		document.forms.caform['name'].value=p['name'];
		document.forms.caform['password'].value=p['password'];
		document.forms.caform['password2'].value=p['password2'];
		document.forms.caform['email'].value=p['email'];
		if(p['language'] == 'en'){
			$("new-site-lang-en").checked = true;
		}else{
			$("new-site-lang-pl").checked = true;
		}
		document.forms.caform['tos'].checked=true;
	}
	OZONE.dom.onDomReady(function(){
		// change links to http://...
		var els = document.getElementsByTagName('a');
		for(var i=0; i<els.length;i++){
			els[i].href = els[i].href.replace(/^https/, 'http');
		}
	}, "dummy-ondomready-block");
}

Wikijump.modules.CreateAccountStep1Module.init();
