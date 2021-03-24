

Wikijump.modules.ASEmailModule = {};

Wikijump.modules.ASEmailModule.listeners = {
	next1: function(e){
		var email = $("ch-email").value;

		if(email == null || email == ''){
			$("email-change-error").innerHTML = "Email must be provided.";
			$("email-change-error").style.display="block";
			return;
		}
		if(!email.match(/^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/)){
			$("email-change-error").innerHTML = "Valid email must be provided.";
			$("email-change-error").style.display="block";
			return;
		}

		var p = new Object();
		p.email = email;
		p.action = "AccountSettingsAction";
		p.event = "changeEmail1";
		OZONE.ajax.requestModule("Account/Settings/Email/ASChangeEmail2Module", p, Wikijump.modules.ASEmailModule.callbacks.next1);

	},

	next2: function(e){
		var evcode = $("ch-evercode").value;

		var p = new Object();
		p.action = "AccountSettingsAction";
		p.event = "changeEmail2";
		p.evercode = evcode;
		OZONE.ajax.requestModule("Account/Settings/Email/ASChangeEmail3Module", p, Wikijump.modules.ASEmailModule.callbacks.next2);

	}
}

Wikijump.modules.ASEmailModule.callbacks = {
	next1: function(r){
		if(r.status == "form_error"){
			$("email-change-error").innerHTML = r.message;
			$("email-change-error").style.display="block";
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}

		$("email-change-area").innerHTML = r.body;
	},

	next2: function(r){
		if(r.status == "form_error"){
			$("email-change-error").innerHTML = r.message;
			$("email-change-error").style.display="block";
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}
		$("email-change-area").innerHTML = r.body;
		$("ech-note").style.display = "none";
		$("ch-el01").style.display = "none";

	}

}
