

Wikijump.modules.LoginModule3 = {};

Wikijump.modules.LoginModule3.listeners = {
	loginClick: function(e){

		YAHOO.util.Event.preventDefault(e);
		var p = OZONE.utils.formToArray("login-form");
		// pre-check:
		var welcome = OZONE.utils.getCookie('welcome');
		if((welcome == null && p['name'] == '') || p['password'] == ''){
			var message="Please fill the login form.";
			$('loginerror').innerHTML = message;
			$("login-head").style.display = "none";
			$('loginerror').style.display="block";
			return;
		}
		if(welcome){
			p['welcome'] = welcome;

		}

		$("login-buttons").style.display="none";
		$("login-progress").style.display="block";

		var rsa = new RSAKey();
		rsa.setPublic(Wikijump.vars.rsakey, "10001");
		p['name'] = linebrk(hex2b64(rsa.encrypt(Wikijump.vars.loginSeed+p['loginName'])),64);
		p['password'] = linebrk(hex2b64(rsa.encrypt(Wikijump.vars.loginSeed+p['password'])),64);

		p.action = "LoginAction";
		p.event = "login";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.LoginModule3.callbacks.loginClick);

	},

	cancel: function(e){
		var p = new Object();
		p.action = "LoginAction";
		p.event = "loginCancel";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.LoginModule3.callbacks.cancel);

	},

	namePress: function(e){
		var chcode = YAHOO.util.Event.getCharCode(e);
		if((chcode == 13) && $('login-form-name').value.length>0 ){
			YAHOO.util.Event.stopEvent(e);
			$('login-form-password').focus();
		}
	}

}

Wikijump.modules.LoginModule3.callbacks = {
	loginClick: function(r){
		if(r.status == 'login_invalid'){
			$("login-head").style.display = "none";
			$("loginerror").innerHTML=r.message;
			$("loginerror").style.display = "block";

			$("login-buttons").style.display="block";
			$("login-progress").style.display="none";
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}
		setTimeout('top.location.href="'+Wikijump.vars.backUrl+'"', 1000);

	},

	cancel: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		OZONE.dialog.cleanAll();
	}
}

Wikijump.modules.LoginModule3.init = function(){
	if($('login-form-name') &&$('login-form-name').type=="text"){
		$('login-form-name').focus();
		YAHOO.util.Event.addListener($('login-form-name'), 'keypress', Wikijump.modules.LoginModule3.listeners.namePress);
	}else{
		$('login-form-password').focus();
	}
	YAHOO.util.Event.addListener("login-form", 'submit', Wikijump.modules.LoginModule3.listeners.loginClick);
}

Wikijump.modules.LoginModule3.init();
