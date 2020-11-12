

Wikijump.modules.PageRateModule = {};

Wikijump.modules.PageRateModule.listeners = {
	showWho: function(e, pageId){
		var p = new Object();
		p.pageId = WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule("pagerate/WhoRatedPageModule", p, Wikijump.modules.PageRateModule.callbacks.showWho);
	}
}

Wikijump.modules.PageRateModule.callbacks = {
	showWho: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("who-rated-page-area").innerHTML = r.body;
		OZONE.visuals.scrollTo($("who-rated-page-area"));
		Wikijump.render.fixAvatarHover($("who-rated-page-area"));
	}
}
