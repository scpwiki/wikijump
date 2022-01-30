

Wikijump.modules.AccountDeletedSitesModule = {};

Wikijump.modules.AccountDeletedSitesModule.vars = {};

Wikijump.modules.AccountDeletedSitesModule.listeners = {
	clickRestore: function(e, siteId){
		var sitesData = Wikijump.modules.AccountDeletedSitesModule.vars.sitesData;
		$("as-restore-site-name").innerHTML = sitesData[siteId]['name'];
		$("as-restore-site-unixname").value = sitesData[siteId]['slug'];
		$("as-restore-site-box").style.display = 'block';
		Wikijump.modules.AccountDeletedSitesModule.vars.siteId = siteId;
		OZONE.visuals.scrollTo($("as-restore-site-box"));

	},

	restore: function(e){
		var p = new Object();
		p.siteId = Wikijump.modules.AccountDeletedSitesModule.vars.siteId;
		p.unixName = $("as-restore-site-unixname").value;
		p.action = 'AccountMembershipAction';
		p.event = 'restoreSite';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AccountDeletedSitesModule.callbacks.restore);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Restoring the site...";
		w.show();
	}
}

Wikijump.modules.AccountDeletedSitesModule.callbacks = {
	restore: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The site has been restored.";
		w.show();

		setTimeout('window.location.href="'+HTTP_SCHEMA+"://"+r.unixName+'.'+URL_DOMAIN+'"', 500);
	}
}

Wikijump.modules.AccountDeletedSitesModule.init = function(){
	Wikijump.modules.AccountDeletedSitesModule.vars.sitesData = JSON.parse(OZONE.utils.unescapeHtml($("as-restore-site-data").innerHTML));

}

Wikijump.modules.AccountDeletedSitesModule.init();
