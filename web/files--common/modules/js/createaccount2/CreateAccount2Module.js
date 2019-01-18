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
	cancel: function(e){
		window.location.href='http://'+window.location.hostname;
	},
	
	backClick: function(e){
		OZONE.ajax.requestModule("createaccount/CreateAccount0Module", null, WIKIDOT.modules.CreateAccount2Module.callbacks.backClick);	
	},
	
	nextClick: function(e){
		var p = new Object();
		p.evcode = $("ca-evercode").value;
		p.action = "CreateAccount2Action";
		p.event = "finalize";
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.CreateAccount2Module.callbacks.nextClick);	
	
	}
}
WIKIDOT.modules.CreateAccount2Module.callbacks = {
	nextClick: function(r){
		if(r.status == "invalid_code"){
			$("ca-error-block").innerHTML = r.message;
			$("ca-error-block").style.display = "block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var t2 = new OZONE.dialogs.SuccessBox(); t2.timeout=10000; t2.content="New account created!";t2.show();
		var originalUrl = r.originalUrl;
		if(r.originalUrlForce){
			setTimeout(function(){
				window.location.href = r.originalUrl;
			}, 2000);
		} else {
			setTimeout(function(){
				var url = '/auth:newaccount3';
				if(originalUrl){
					url = url + '?origUrl=' + encodeURIComponent(originalUrl);
				}
				window.location.href = url;
			}, 2000);
		}
	},
	backClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
	}

}
OZONE.dom.onDomReady(function(){		
		// change links to http://...
		var els = document.getElementsByTagName('a');
		for(var i=0; i<els.length;i++){
			els[i].href = els[i].href.replace(/^https/, 'http');
		}
	}, "dummy-ondomready-block");
//YAHOO.util.Event.addListener("next-click", "click", WIKIDOT.modules.AcceptTOSModule.listeners.nextClick);
