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

WIKIDOT.CreateAccountModule = {};

WIKIDOT.CreateAccountModule.listeners = {
	createClick: function(e){
		OZONE.ajax.requestModule("createaccount/AcceptTOSModule", null,WIKIDOT.CreateAccountModule.callbacks.createClick );
	}	
}

WIKIDOT.CreateAccountModule.callbacks = {
	createClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-account-area", response.body);
	}	
}

YAHOO.util.Event.addListener("create-account-button", "click", WIKIDOT.CreateAccountModule.listeners.createClick);;