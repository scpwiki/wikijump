


Wikijump.modules.AcceptTOSModule = {};

Wikijump.modules.AcceptTOSModule.listeners = {

	nextClick: function(e){
		var p = OZONE.utils.formToArray('accept-tos-form');
		p.action="CreateAccountAction";
		p.event = "acceptRules";
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", p, Wikijump.modules.AcceptTOSModule.callbacks.nextClick);

	}
}
Wikijump.modules.AcceptTOSModule.callbacks = {
	nextClick: function(r){
		if(r.status == "must_accept"){
			$("accept-tos-error").innerHTML = r.message;
			$("accept-tos-error").style.display = "block";
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}

}

Wikijump.modules.AcceptTOSModule.init = function(){

}
