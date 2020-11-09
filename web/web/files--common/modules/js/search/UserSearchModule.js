

Wikijump.modules.UserSearchModule = {};

Wikijump.modules.UserSearchModule.listeners = {
	search: function(e){
		var f = OZONE.utils.formToArray("search-form-user");
		var query = f['query'];
		// escape query
		query = encodeURIComponent(query);
		var url = "/search:users";
		url += "/q/"+query;
		window.location.href=url;
		YAHOO.util.Event.preventDefault(e);
	}
}

Wikijump.modules.UserSearchModule.init = function(){
	YAHOO.util.Event.addListener("search-form-user", "submit", Wikijump.modules.UserSearchModule.listeners.search);
}

Wikijump.modules.UserSearchModule.init();
