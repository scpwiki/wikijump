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

WIKIDOT.modules.FlagPageModule = {};

WIKIDOT.modules.FlagPageModule.listeners = {
	setFlag: function(e, flag){
		var p = new Object();
		p.path = window.location.pathname;
		p.action = "AbuseFlagAction";
		p.event = "FlagPage";
		if(flag){
			p.flag = "yes";
			$("flag-page-options-flag").style.display="none";
			$("flag-page-options-unflag").style.display="block";
		}else{
			$("flag-page-options-flag").style.display="block";
			$("flag-page-options-unflag").style.display="none";
		}
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.FlagPageModule.callbacks.setFlag);
		
	}	
	
}

WIKIDOT.modules.FlagPageModule.callbacks = {
	setFlag: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
	}
		
}