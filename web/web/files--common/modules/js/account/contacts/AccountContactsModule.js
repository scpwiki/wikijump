

Wikijump.modules.AccountContactsModule = {};

Wikijump.modules.AccountContactsModule.vars = {};

Wikijump.modules.AccountContactsModule.listeners = {
	showAddForm: function(e){
		if(!Wikijump.modules.AccountContactsModule.vars.addFormInited){
			// init autocomplete now
			var dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users','name', 'user_id']);
			dataSource.scriptQueryParam="q";
			dataSource.scriptQueryAppend = "&module=UserLookupQModule";

			var autoComp = new YAHOO.widget.AutoComplete('user-lookup','user-lookup-list', dataSource);

			autoComp.minQueryLength = 2;
			autoComp.queryDelay = 0.5;
			autoComp.forceSelection = true;
			autoComp.itemSelectEvent.subscribe(function(sType, args){
				var userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/,"$1");
				var userName = args[1].getElementsByTagName('div').item(0).innerHTML;
				Wikijump.modules.AccountContactsModule.listeners.selectUser(userId, userName);
			});

			autoComp.formatResult = function(aResultItem, sQuery) {
				var name = aResultItem[0];
				var userId = aResultItem[1];
				if(name!= null){
					return '<div id="user-autocomplete-'+userId+'">'+name+'</div>';
				} else {
					return "";
				}

			}

			Wikijump.modules.AccountContactsModule.vars.addFormInited = true;
		}
		$("show-add-contact-button").style.display = "none";
		$("add-contact-user-div").style.display = "block";
		OZONE.visuals.scrollTo("add-contact-user-div");
	},
	cancelAdd: function(e){
		// resets the forms?
		$("show-add-contact-button").style.display = "block";
		$("add-contact-user-div").style.display = "none";
		$("user-lookup").value="";
		Wikijump.modules.AccountContactsModule.listeners.changeUser(null);

	},
	selectUser: function(userId, userName){
		var userString = Wikijump.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		Wikijump.modules.AccountContactsModule.vars.currentUserId = userId;
	},
	changeUser: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		Wikijump.modules.AccountContactsModule.vars.currentUserId = null;
	},

	addContact: function(e){
		if(Wikijump.modules.AccountContactsModule.vars.currentUserId == null){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You must select a valid user to add.";
			w.show();
			return;
		}
		var p = new Object();
		OZONE.ajax.requestModule('userinfo/UserAddToContactsModule', {userId: Wikijump.modules.AccountContactsModule.vars.currentUserId}, Wikijump.modules.AccountContactsModule.callbacks.addContact);
	},

	showBack: function(e){
		OZONE.ajax.requestModule("account/contacts/AccountBackContactsModule", null, Wikijump.modules.AccountContactsModule.callbacks.showBack);
	},

	removeContact: function(e, userId){
		var p = new Object();
		p.action = "ContactsAction";
		p.event = "removeContact";
		p.userId = userId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountContactsModule.callbacks.removeContact);
	},

	refresh: function(e){
		Wikijump.modules.AccountModule.utils.loadModule("am-contacts");
	}
}

Wikijump.modules.AccountContactsModule.callbacks = {
	showBack: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("back-contacts-list").innerHTML = r.body;
	},
	addContact: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	},
	removeContact: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User removed from contacts";
		w.show();
		Wikijump.modules.AccountContactsModule.listeners.refresh();
	}
}
