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

WIKIDOT.modules.UserSearchModule = {};

WIKIDOT.modules.UserSearchModule.listeners = {
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

WIKIDOT.modules.UserSearchModule.init = function(){
	YAHOO.util.Event.addListener("search-form-user", "submit", WIKIDOT.modules.UserSearchModule.listeners.search);
}

WIKIDOT.modules.UserSearchModule.init();