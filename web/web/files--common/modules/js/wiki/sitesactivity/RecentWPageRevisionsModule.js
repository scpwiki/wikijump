

Wikijump.modules.RecentWPageRevisionsModule = {};

Wikijump.modules.RecentWPageRevisionsModule.vars = {};

Wikijump.modules.RecentWPageRevisionsModule.listeners = {
	update: function(){
		OZONE.ajax.requestModule('Wiki/SitesActivity/RecentWPageRevisionsModule', null, Wikijump.modules.RecentWPageRevisionsModule.callbacks.update);
	}
}

Wikijump.modules.RecentWPageRevisionsModule.callbacks = {
	update: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var body = r.body.replace(/<div[^>]*>/, '').replace(/<\/div>\s*$/, '');

		if(body != $("recent-w-page-revisions").innerHTML){

			$("recent-w-page-revisions").innerHTML = body;

//			$("recent-w-page-revisions")

		}
	}
}

Wikijump.modules.RecentWPageRevisionsModule.init = function(){
	setTimeout('Wikijump.modules.RecentWPageRevisionsModule.listeners.update()', 20000);
}

//Wikijump.modules.RecentWPageRevisionsModule.init();
