

Wikijump.modules.MembershipApplyModule = {};

Wikijump.modules.MembershipApplyModule.listeners = {
	apply: function(e){
		var parms = OZONE.utils.formToArray("membership-by-apply-form");
		parms['action'] = "MembershipApplyAction";
		parms['event'] = "apply";
		OZONE.ajax.requestModule("membership/MembershipApplySuccessModule", parms, Wikijump.modules.MembershipApplyModule.callbacks.apply);
	}
}

Wikijump.modules.MembershipApplyModule.callbacks = {
	apply: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessDialog();
		w.content = "Your application has been sent and now awaits to be processed by " +
				"the site administrators.";
		w.addButtonListener('close message', function(){window.location.reload()});
		w.show();

		// check if any errors:
	}

}

Wikijump.modules.MembershipApplyModule.init = function(){
	OZONE.dom.onDomReady(function(){
		if($("membership-by-apply-text")){
			YAHOO.util.Event.addListener("mba-apply", "click", Wikijump.modules.MembershipApplyModule.listeners.apply);
			var limiter = new OZONE.forms.lengthLimiter("membership-by-apply-text", "membership-by-apply-text-left", 200);
		}
	}, "dummy-ondomready-block");
}

Wikijump.modules.MembershipApplyModule.init();
