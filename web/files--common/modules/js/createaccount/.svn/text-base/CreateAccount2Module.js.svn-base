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

WIKIDOT.modules.CreateAccount2Module = {};

WIKIDOT.modules.CreateAccount2Module.listeners = {
	cancelClick: function(e){
		window.location.href="/";
	},
	
	backClick: function(e){
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", null, WIKIDOT.modules.CreateAccount2Module.callbacks.backClick);	
	},
	
	nextClick: function(e){
		var p = new Object();
		p.evcode = $("ca-evercode").value;
		p.action = "CreateAccountAction";
		p.event = "finalize";
		OZONE.ajax.requestModule("createaccount/CreateAccount3Module", p, WIKIDOT.modules.CreateAccount2Module.callbacks.nextClick);	
	
	}
}
WIKIDOT.modules.CreateAccount2Module.callbacks = {
	nextClick: function(r){
		if(r.status == "invalid_code"){
			$("ca-error-block").innerHTML = r.message;
			$("ca-error-block").style.display = "block";
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}

		if(!WIKIREQUEST.createAccountSkipCongrats){
			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.show();
		}else{
			window.location.reload()
		}
	},
	backClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}

}

//YAHOO.util.Event.addListener("next-click", "click", WIKIDOT.modules.AcceptTOSModule.listeners.nextClick);
