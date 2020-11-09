

Wikijump.modules.SearchModule = {};

Wikijump.modules.SearchModule.listeners = {
	search: function(e){
		var f = OZONE.utils.formToArray("search-form");
		var query = f['query'];
		var area = f['area'];
		// escape query
		query = encodeURIComponent(query);
		var url = "/search:site";
		if(area && area != ''){
			url += "/a/"+area;
		}
		url += "/q/"+query;
		window.location.href=url;
		YAHOO.util.Event.preventDefault(e);
	}
}

Wikijump.modules.SearchModule.init = function(){
	YAHOO.util.Event.addListener("search-form", "submit", Wikijump.modules.SearchModule.listeners.search);
}

Wikijump.modules.SearchModule.init();
