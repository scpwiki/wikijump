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

WIKIDOT.modules.CreateAccount1Module = {};

WIKIDOT.modules.CreateAccount1Module.listeners = {
	cancelClick: function(e){
		OZONE.dialog.cleanAll();
	},
	
	backClick: function(e){
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", null, WIKIDOT.modules.CreateAccount1Module.callbacks.backClick);	
	},
	
	nextClick: function(e){
		var p = new Object();
		p.action = "CreateAccountAction";
		p.event = "sendEmailVer";
		OZONE.ajax.requestModule("createaccount/CreateAccount2Module", p, WIKIDOT.modules.CreateAccount1Module.callbacks.nextClick);	
	
	}
}
WIKIDOT.modules.CreateAccount1Module.callbacks = {
	nextClick: function(r){
		if(r.status == "email_failed"){
			$("ca-error-block").innerHTML = r.message;
			$("ca-error-block").style.display = "block";
			OZONE.dialog.factory.boxcontainer().centerContent();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}	,
	backClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}

}

WIKIDOT.modules.CreateAccount1Module.init = function(){
	var p = WIKIDOT.modules.CreateAccountModule.vars.formData;
	if(p == null){
		alert("Registration flow error.");
		window.location.reload();
	}
	$("ca-field-name").innerHTML = p['name'];
	$("ca-field-email").innerHTML = p['email'];	
	
}
WIKIDOT.modules.CreateAccount1Module.init();

//YAHOO.util.Event.addListener("next-click", "click", WIKIDOT.modules.AcceptTOSModule.listeners.nextClick);
