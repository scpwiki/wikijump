

Wikijump.modules.SearchAllModulee = {};

Wikijump.modules.SearchAllModulee.listeners = {
	search: function(e){
		var f = OZONE.utils.formToArray("search-form-all");
		var query = f['query'];
		var area = f['area'];
		// escape query
		query = encodeURIComponent(query);
		var url = "/search:all";
		if(area && area != ''){
			url += "/a/"+area;
		}
		url += "/q/"+query;
		window.location.href=url;
		YAHOO.util.Event.preventDefault(e);
	}
}

Wikijump.modules.SearchAllModulee.init = function(){
	YAHOO.util.Event.addListener("search-form-all", "submit", Wikijump.modules.SearchAllModulee.listeners.search);
}

Wikijump.modules.SearchAllModulee.init();
