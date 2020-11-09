

Wikijump.modules.ManageSitePageAbuseModule = {};

Wikijump.modules.ManageSitePageAbuseModule.listeners = {
	clear: function(e, path){
		var p = new Object();
		p.action = "ManageSiteAbuseAction";
		p.event = "clearPageFlags";
		p.path = path;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSitePageAbuseModule.callbacks.clear);
	}

}

Wikijump.modules.ManageSitePageAbuseModule.callbacks = {
	clear: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Flags cleared";
		w.show();

		Wikijump.modules.ManagerSiteModule.utils.loadModule("sm-abuse-page");
	}
}
