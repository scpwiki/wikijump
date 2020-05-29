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

WIKIDOT.modules.CreateSite0Module = {};

WIKIDOT.modules.CreateSite0Module.listeners = {
	cancelClick: function(e){
		window.location.href="/";
	},
	
	nextClick: function(e){
		params = OZONE.utils.formToArray('new-site1');
		OZONE.ajax.requestModule("createsite/CreateSite1Module", params, WIKIDOT.modules.CreateSite0Module.callbacks.nextClick);	
	
	},
	
	licenceSelect: function(e){
		val = document.getElementById("licence-select").value;
		if(val == 'other'){
			// show row
			document.getElementById('other-licence-row').style.display="";
		} else {
			document.getElementById('other-licence-row').style.display="none";
		}
	}
}
WIKIDOT.modules.CreateSite0Module.callbacks = {
	nextClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);
		
	}	

}

YAHOO.util.Event.addListener("licence-select", "change", WIKIDOT.modules.CreateSite0Module.listeners.licenceSelect);

WIKIDOT.modules.CreateSite0Module.listeners.licenceSelect(null); // to initialize
