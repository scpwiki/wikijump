

Wikijump.modules.NewWikiWidgetModule = {}

Wikijump.modules.NewWikiWidgetModule.listeners = {
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
		p.action = 'Wiki/Special/NewWikiWidgetAction';
		p.event = 'newWiki';
		p.siteName = siteName;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.NewWikiWidgetModule.callbacks.submitCallback);
	}
}

Wikijump.modules.NewWikiWidgetModule.callbacks = {
	submitCallback: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		// seems fine.
		window.location.href='/new-site/address/'+r.unixName;
	}
}
