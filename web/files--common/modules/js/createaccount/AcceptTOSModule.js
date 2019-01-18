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


WIKIDOT.modules.AcceptTOSModule = {};

WIKIDOT.modules.AcceptTOSModule.listeners = {

	nextClick: function(e){
		var p = OZONE.utils.formToArray('accept-tos-form');
		p.action="CreateAccountAction";
		p.event = "acceptRules";
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", p, WIKIDOT.modules.AcceptTOSModule.callbacks.nextClick);	
	
	}
}
WIKIDOT.modules.AcceptTOSModule.callbacks = {
	nextClick: function(r){
		if(r.status == "must_accept"){
			$("accept-tos-error").innerHTML = r.message;
			$("accept-tos-error").style.display = "block";
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}	

}

WIKIDOT.modules.AcceptTOSModule.init = function(){
	
}
