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

WIKIDOT.modules.MembershipApplyModule = {};

WIKIDOT.modules.MembershipApplyModule.listeners = {
	apply: function(e){
		var parms = OZONE.utils.formToArray("membership-by-apply-form");
		parms['action'] = "MembershipApplyAction";
		parms['event'] = "apply";
		OZONE.ajax.requestModule("membership/MembershipApplySuccessModule", parms, WIKIDOT.modules.MembershipApplyModule.callbacks.apply);
	}	
}

WIKIDOT.modules.MembershipApplyModule.callbacks = {
	apply: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "Your application has been sent and now awaits to be processed by " +
				"the site administrators.";
		w.addButtonListener('close message', function(){window.location.reload()});
		w.show();

		// check if any errors:
	}	
	
}

WIKIDOT.modules.MembershipApplyModule.init = function(){
	OZONE.dom.onDomReady(function(){		
		if($("membership-by-apply-text")){
			YAHOO.util.Event.addListener("mba-apply", "click", WIKIDOT.modules.MembershipApplyModule.listeners.apply);
			var limiter = new OZONE.forms.lengthLimiter("membership-by-apply-text", "membership-by-apply-text-left", 200);
		}
	}, "dummy-ondomready-block");
}

WIKIDOT.modules.MembershipApplyModule.init();
