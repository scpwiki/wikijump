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

WIKIDOT.modules.NewWikiWidgetModule = {}

WIKIDOT.modules.NewWikiWidgetModule.listeners = {
	submit: function(event){
		if(YAHOO.util.Dom.hasClass("new-wiki-widget-site-name", 'empty')){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You need to enter a valid web address for your new wiki.";
			w.show();
			return;
		}
		var siteName = $("new-wiki-widget-site-name").value;
		siteName = siteName.replace(/^\s+/, '').replace(/\s+$/,'');
		// validate a bit
		if(siteName.length <3){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You need to provide the web address for you wiki and it should be at least 3 characters long.";
			w.show();
			return;
		}
		var p = new Object();
		p.action = 'wiki/special/NewWikiWidgetAction';
		p.event = 'newWiki';
		p.siteName = siteName;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.NewWikiWidgetModule.callbacks.submitCallback);
	}
}

WIKIDOT.modules.NewWikiWidgetModule.callbacks = {
	submitCallback: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		// seems fine.
		window.location.href='/new-site/address/'+r.unixName;
	}
}