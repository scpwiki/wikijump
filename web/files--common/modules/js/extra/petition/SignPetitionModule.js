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

WIKIDOT.modules.SignPetitionModule = {};

WIKIDOT.modules.SignPetitionModule.listeners = {
	
	sign: function(e){
		var p = OZONE.utils.formToArray("sign-petition-form");
		p.action = "extra/petition/PetitionAction";
		p.event = "sign";
		p.petitionUrl = WIKIREQUEST.info.pageUnixName;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.SignPetitionModule.callbacks.sign );
	},
	
	confirmSignature: function(e, campaignId, hash){
		var p = new Object();
		p.action = "extra/petition/PetitionAction";
		p.event = "confirm";
		p.hash = hash;
		p.campaignId = campaignId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.SignPetitionModule.callbacks.confirmSignature);
		
	},
	
	cancelSignature: function(e, campaignId, hash){
		var p = new Object();
		p.action = "extra/petition/PetitionAction";
		p.event = "cancel";
		p.hash = hash;
		p.campaignId = campaignId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.SignPetitionModule.callbacks.confirmSignature);
		
	}
}

WIKIDOT.modules.SignPetitionModule.callbacks = {
	sign: function(r){
		var form = $('sign-petition-form');
		var trs = form.getElementsByTagName('tr');
		for(var i=0; i<trs.length; i++){
			YAHOO.util.Dom.removeClass(trs[i],'invalid-value-row');
		}
		
		if(r.status == 'form_errors'){
			$("sign-petition-error-box").innerHTML = r.message;
			$("sign-petition-error-box").style.display="block";
			var errors = r.errors;
			
			for(var n in errors){
				var row = $('sign-petition-row'+'-'+n);
				YAHOO.util.Dom.addClass(row,'invalid-value-row');
				var errorDiv = YAHOO.util.Dom.getElementsByClassName('field-error-message', 'div', row)[0];
				errorDiv.innerHTML = errors[n];
			}
			OZONE.dialog.cleanAll();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		$("sign-petition-box").innerHTML = r.body;
	},
	
	confirmSignature: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		alert(r.thankYouPage);
		if(r.thankYouPage){
			window.location.href='/'+r.thankYouPage;
		}else{
			$("sign-petition-box").innerHTML = r.body;
		}
	}
	
}

WIKIDOT.modules.SignPetitionModule.init = function(){
	if(window.location.pathname.match(/\/confirm\//)){
		OZONE.dom.onDomReady(function(){
			setTimeout('OZONE.visuals.scrollTo("sign-petition-box");', 100);
		}, "dummy-ondomready-block");
	}
}

WIKIDOT.modules.SignPetitionModule.init();
