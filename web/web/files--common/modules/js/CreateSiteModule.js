

Wikijump.CreateSiteModule = {};

Wikijump.CreateSiteModule.listeners = {
	createClick: function(e){
		OZONE.ajax.requestModule("createsite/CreateSite0Module", null,Wikijump.CreateSiteModule.callbacks.createClick );
	}
}

Wikijump.CreateSiteModule.callbacks = {
	createClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);
	}
}

YAHOO.util.Event.addListener("create-site-button", "click", Wikijump.CreateSiteModule.listeners.createClick);;
