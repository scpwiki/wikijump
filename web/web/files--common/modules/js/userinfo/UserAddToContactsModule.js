

Wikijump.modules.UserAddToContacts = {};

Wikijump.modules.UserAddToContacts.listeners = {
	addContact: function(event, userId)	{
		var p = new Object();
		p.action = "ContactsAction";
		p.event = "addContact";
		p.userId = userId;

		OZONE.ajax.requestModule(null, p, Wikijump.modules.UserAddToContacts.callbacks.addContact);
	}
}

Wikijump.modules.UserAddToContacts.callbacks = {
	addContact: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User added to contacts";
		w.show();
		if(Wikijump.modules.AccountContactsModule){
			Wikijump.modules.AccountContactsModule.listeners.refresh();
		}
	}

}
