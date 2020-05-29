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

WIKIDOT.modules.SearchAllModulee = {};

WIKIDOT.modules.SearchAllModulee.listeners = {
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

WIKIDOT.modules.SearchAllModulee.init = function(){
	YAHOO.util.Event.addListener("search-form-all", "submit", WIKIDOT.modules.SearchAllModulee.listeners.search);
}

WIKIDOT.modules.SearchAllModulee.init();