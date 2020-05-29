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

WIKIDOT.modules.AccountContactsModule = {};

WIKIDOT.modules.AccountContactsModule.vars = {};

WIKIDOT.modules.AccountContactsModule.listeners = {
	showAddForm: function(e){
		if(!WIKIDOT.modules.AccountContactsModule.vars.addFormInited){
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
				WIKIDOT.modules.AccountContactsModule.listeners.selectUser(userId, userName);
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
			
			WIKIDOT.modules.AccountContactsModule.vars.addFormInited = true;
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
		WIKIDOT.modules.AccountContactsModule.listeners.changeUser(null);
		
	},
	selectUser: function(userId, userName){
		var userString = WIKIDOT.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		WIKIDOT.modules.AccountContactsModule.vars.currentUserId = userId;
	},
	changeUser: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		WIKIDOT.modules.AccountContactsModule.vars.currentUserId = null;
	},
	
	addContact: function(e){
		if(WIKIDOT.modules.AccountContactsModule.vars.currentUserId == null){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You must select a valid user to add.";
			w.show();
			return;
		}
		var p = new Object();
		OZONE.ajax.requestModule('userinfo/UserAddToContactsModule', {userId: WIKIDOT.modules.AccountContactsModule.vars.currentUserId}, WIKIDOT.modules.AccountContactsModule.callbacks.addContact);
	},
	
	showBack: function(e){
		OZONE.ajax.requestModule("account/contacts/AccountBackContactsModule", null, WIKIDOT.modules.AccountContactsModule.callbacks.showBack);
	},

	removeContact: function(e, userId){
		var p = new Object();
		p.action = "ContactsAction";
		p.event = "removeContact";
		p.userId = userId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountContactsModule.callbacks.removeContact);
	},
	
	refresh: function(e){
		WIKIDOT.modules.AccountModule.utils.loadModule("am-contacts");
	}
}

WIKIDOT.modules.AccountContactsModule.callbacks = {
	showBack: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;} 
		$("back-contacts-list").innerHTML = r.body;
	},
	addContact: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();	
	},
	removeContact: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User removed from contacts";
		w.show();
		WIKIDOT.modules.AccountContactsModule.listeners.refresh();
	}
}
