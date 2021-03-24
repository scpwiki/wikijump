

Wikijump.modules.ManagerSiteDeleteModule = {};

Wikijump.modules.ManagerSiteDeleteModule.vars = {
	currentCategory: null
}

Wikijump.modules.ManagerSiteDeleteModule.listeners = {
	deleteSite: function(event){
		var p = new Object();

		OZONE.ajax.requestModule("ManageSite/ManageSiteDelete2Module", p, Wikijump.modules.ManagerSiteDeleteModule.callbacks.deleteSite);
	},
	deleteSite2: function(event){
		var p = new Object();
		p.action = "ManageSiteAction";
		p.event = "DeleteSite";

		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManagerSiteDeleteModule.callbacks.deleteSite2);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Deleting the site...";
		w.show();
	}
}

Wikijump.modules.ManagerSiteDeleteModule.callbacks = {
	deleteSite: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("sm-delete-box").innerHTML = r.body;

	},
	deleteSite2: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The site has been deleted.";
		setTimeout('window.location.href="'+HTTP_SCHEMA+"://"+URL_DOMAIN+'/account:you/start/deletedsites"', 1000);
		w.show();
	}
}
