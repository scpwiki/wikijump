

Wikijump.modules.AccountNotificationsModule = {};

Wikijump.modules.AccountNotificationsModule.listeners = {
	loadList: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("Account/AccountNotificationsListModule", p, Wikijump.modules.AccountNotificationsModule.callbacks.loadList);
	}
}

Wikijump.modules.AccountNotificationsModule.callbacks = {
	loadList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("notifications-area").innerHTML = r.body;
		OZONE.utils.formatDates($("notifications-area"));
	}
}

Wikijump.modules.AccountNotificationsModule.init = function(){
	loadList(1);
}

function loadList(pageNo){
}

Wikijump.modules.AccountNotificationsModule.init();
