

Wikijump.modules.ManageSiteNotificationsModule = {};

Wikijump.modules.ManageSiteNotificationsModule.listeners = {
	loadList: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("managesite/ManageSiteNotificationsListModule", p, Wikijump.modules.ManageSiteNotificationsModule.callbacks.loadList);
	}
}

Wikijump.modules.ManageSiteNotificationsModule.callbacks = {
	loadList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("notifications-area").innerHTML = r.body;
		OZONE.utils.formatDates($("notifications-area"));
	}
}

Wikijump.modules.ManageSiteNotificationsModule.init = function(){
	loadList(1);
}

function loadList(pageNo){
}

Wikijump.modules.ManageSiteNotificationsModule.init();
