

Wikijump.modules.CreateAccount1Module = {};

Wikijump.modules.CreateAccount1Module.listeners = {
	cancelClick: function(e){
		OZONE.dialog.cleanAll();
	},

	backClick: function(e){
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", null, Wikijump.modules.CreateAccount1Module.callbacks.backClick);
	},

	nextClick: function(e){
		var p = new Object();
		p.action = "CreateAccountAction";
		p.event = "sendEmailVer";
		OZONE.ajax.requestModule("createaccount/CreateAccount2Module", p, Wikijump.modules.CreateAccount1Module.callbacks.nextClick);

	}
}
Wikijump.modules.CreateAccount1Module.callbacks = {
	nextClick: function(r){
		if(r.status == "email_failed"){
			$("ca-error-block").innerHTML = r.message;
			$("ca-error-block").style.display = "block";
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}	,
	backClick: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}

}

Wikijump.modules.CreateAccount1Module.init = function(){
	var p = Wikijump.modules.CreateAccountModule.vars.formData;
	if(p == null){
		alert("Registration flow error.");
		window.location.reload();
	}
	$("ca-field-name").innerHTML = p['name'];
	$("ca-field-email").innerHTML = p['email'];

}
Wikijump.modules.CreateAccount1Module.init();

//YAHOO.util.Event.addListener("next-click", "click", Wikijump.modules.AcceptTOSModule.listeners.nextClick);
