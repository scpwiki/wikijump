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

WIKIDOT.modules.FlagAnonymousModule = {};

WIKIDOT.modules.FlagAnonymousModule.listeners = {
	setFlag: function(e, userString, flag){
		var p = new Object();
		p.path = window.location.pathname;
		p.action = "AbuseFlagAction";
		p.event = "flagAnonymous";
		p.userString = userString;
		if(flag){
			p.flag = "yes";

			$("flag-user-options-flag").style.display="none";
			$("flag-user-options-unflag").style.display="block";
		}else{
			$("flag-user-options-flag").style.display="block";
			$("flag-user-options-unflag").style.display="none";
		}
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.FlagAnonymousModule.callbacks.setFlag);
		
	}	
	
}

WIKIDOT.modules.FlagAnonymousModule.callbacks = {
	setFlag: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
	}
		
}
