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

WIKIDOT.modules.AccountDeletedSitesModule = {};

WIKIDOT.modules.AccountDeletedSitesModule.vars = {};

WIKIDOT.modules.AccountDeletedSitesModule.listeners = {
	clickRestore: function(e, siteId){
		var sitesData = WIKIDOT.modules.AccountDeletedSitesModule.vars.sitesData;
		$("as-restore-site-name").innerHTML = sitesData[siteId]['name'];
		$("as-restore-site-unixname").value = sitesData[siteId]['unix_name'];
		$("as-restore-site-box").style.display = 'block';
		WIKIDOT.modules.AccountDeletedSitesModule.vars.siteId = siteId;
		OZONE.visuals.scrollTo($("as-restore-site-box"));
		
	},
	
	restore: function(e){
		var p = new Object();
		p.siteId = WIKIDOT.modules.AccountDeletedSitesModule.vars.siteId;
		p.unixName = $("as-restore-site-unixname").value;
		p.action = 'AccountMembershipAction';
		p.event = 'restoreSite';
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.AccountDeletedSitesModule.callbacks.restore);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Restoring the site...";
		w.show();
	}
}

WIKIDOT.modules.AccountDeletedSitesModule.callbacks = {
	restore: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The site has been restored.";
		w.show();
		
		setTimeout('window.location.href="http://'+r.unixName+'.'+URL_DOMAIN+'"', 500);
	}
}

WIKIDOT.modules.AccountDeletedSitesModule.init = function(){
	WIKIDOT.modules.AccountDeletedSitesModule.vars.sitesData = JSON.parse(OZONE.utils.unescapeHtml($("as-restore-site-data").innerHTML));

}

WIKIDOT.modules.AccountDeletedSitesModule.init();
