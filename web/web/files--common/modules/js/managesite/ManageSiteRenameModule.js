

Wikijump.modules.ManagerSiteRenameModule = {};

Wikijump.modules.ManagerSiteRenameModule.vars = {
	currentCategory: null
}

Wikijump.modules.ManagerSiteRenameModule.listeners = {
	renameSite: function(event){
		var p = new Object();
		p.unixName = $("sm-rename-site-unixname").value;
		p.action = 'ManageSiteAction';
		p.event = 'renameSite';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManagerSiteRenameModule.callbacks.renameSite);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Changing the URL...";
		w.show();
	}
}

Wikijump.modules.ManagerSiteRenameModule.callbacks = {
	renameSite: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The URL has been changed.";
		w.show();

		setTimeout('window.location.href="'+HTTP_SCHEMA+"://"+r.unixName+'.'+URL_DOMAIN+'"', 500);

	}

}
