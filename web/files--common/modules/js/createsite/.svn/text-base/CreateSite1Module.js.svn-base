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

WIKIDOT.modules.CreateSite1Module = {};

WIKIDOT.modules.CreateSite1Module.listeners = {
	cancelClick: function(e){
		window.location.href="/";
	},
	
	nextClick: function(e){
		parms = new Array();
		parms['action']='CreateSiteAction';
		parms['event']='finalize';
		OZONE.ajax.requestModule("createsite/CreateSite2Module", parms, WIKIDOT.modules.CreateSite1Module.callbacks.nextClick);	
	
	},
	
	backClick: function(e){
		OZONE.ajax.requestModule("createsite/CreateSite0Module", null, WIKIDOT.modules.CreateSite1Module.callbacks.backClick);	
	
	}
	
}
WIKIDOT.modules.CreateSite1Module.callbacks = {
	nextClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);
		
	},
	backClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);
		
	}	

}
