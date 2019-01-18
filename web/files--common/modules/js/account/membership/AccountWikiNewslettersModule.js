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

WIKIDOT.modules.AccountWikiNewslettersModule = {};

WIKIDOT.modules.AccountWikiNewslettersModule.listeners = {
	checkAll: function(e, value){
		var inps = YAHOO.util.Dom.getElementsByClassName("receive-newsletter", "input", "receive-wiki-newsletters-form");
		for(var i=0; i<inps.length; i++){
			inps[i].checked = value;
		}		
	},
	
	saveDefault: function(e){
		var p = new Object();
		p.action = 
	}

}

WIKIDOT.modules.AccountWikiNewslettersModule.callbacks = {
	saveDefault: function(r){
		
	}	
}
