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

WIKIDOT.modules.MembershipByPasswordModule = {};

WIKIDOT.modules.MembershipByPasswordModule.listeners = {
	apply: function(e){
		var parms = OZONE.utils.formToArray("membership-by-password-form");
		parms['action'] = "MembershipApplyAction";
		parms['event'] = "applyByPassword";
		OZONE.ajax.requestModule("membership/MembershipByPasswordResultModule", parms, WIKIDOT.modules.MembershipByPasswordModule.callbacks.apply);
	}	
}

WIKIDOT.modules.MembershipByPasswordModule.callbacks = {
	apply: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "Congratulations! You are now a member of this site!";
			w.addButtonListener('close message', function(){window.location.reload()});
			w.show();
		} else {
			if(!WIKIDOT.utils.handleError(r)) {return;}	
		}
		return;
		
	}	
	
}

WIKIDOT.modules.MembershipByPasswordModule.init = function(){
}

WIKIDOT.modules.MembershipByPasswordModule.init();
