

Wikijump.modules.UserInfoWinModule = {};

Wikijump.modules.UserInfoWinModule.listeners = {
	flagUser: function(e, userId){
		OZONE.ajax.requestModule('report/FlagUserModule', {targetUserId: userId}, Wikijump.modules.UserInfoWinModule.callbacks.flagUser);

	},
	addContact: function(e, userId){
		OZONE.ajax.requestModule('userinfo/UserAddToContactsModule', {userId: userId}, Wikijump.modules.UserInfoWinModule.callbacks.addContact);
	}

}

Wikijump.modules.UserInfoWinModule.callbacks = {
	flagUser: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	},
	addContact: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}
}
