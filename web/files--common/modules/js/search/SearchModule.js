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

WIKIDOT.modules.SearchModule = {};

WIKIDOT.modules.SearchModule.listeners = {
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

WIKIDOT.modules.SearchModule.init = function(){
	YAHOO.util.Event.addListener("search-form", "submit", WIKIDOT.modules.SearchModule.listeners.search);
}

WIKIDOT.modules.SearchModule.init();