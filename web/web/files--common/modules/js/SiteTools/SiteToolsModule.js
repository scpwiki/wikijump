

Wikijump.modules.SiteToolsModule = {};

Wikijump.modules.SiteToolsModule.listeners = {
	wantedPages: function(e){
		OZONE.ajax.requestModule("SiteTools/WantedPagesModule", null, Wikijump.modules.SiteToolsModule.callbacks.setContent);
	},
	orphanedPages: function(e){
		OZONE.ajax.requestModule("SiteTools/OrphanedPagesModule", null, Wikijump.modules.SiteToolsModule.callbacks.setContent);
	}
}

Wikijump.modules.SiteToolsModule.callbacks = {
	setContent: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("st-action-area").innerHTML = r.body;
		OZONE.visuals.scrollTo("st-action-area");
	}
}
