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

WIKIDOT.CreateSiteModule = {};

WIKIDOT.CreateSiteModule.listeners = {
	createClick: function(e){
		OZONE.ajax.requestModule("createsite/CreateSite0Module", null,WIKIDOT.CreateSiteModule.callbacks.createClick );
	}	
}

WIKIDOT.CreateSiteModule.callbacks = {
	createClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);
	}	
}

YAHOO.util.Event.addListener("create-site-button", "click", WIKIDOT.CreateSiteModule.listeners.createClick);;