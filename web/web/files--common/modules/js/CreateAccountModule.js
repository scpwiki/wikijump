

Wikijump.CreateAccountModule = {};

Wikijump.CreateAccountModule.listeners = {
	createClick: function(e){
		OZONE.ajax.requestModule("CreateAccount/AcceptTOSModule", null,Wikijump.CreateAccountModule.callbacks.createClick );
	}
}

Wikijump.CreateAccountModule.callbacks = {
	createClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-account-area", response.body);
	}
}

YAHOO.util.Event.addListener("create-account-button", "click", Wikijump.CreateAccountModule.listeners.createClick);;
