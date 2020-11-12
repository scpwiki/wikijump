

Wikijump.modules.MembershipByPasswordModule = {};

Wikijump.modules.MembershipByPasswordModule.listeners = {
	apply: function(e){
		var parms = OZONE.utils.formToArray("membership-by-password-form");
		parms['action'] = "MembershipApplyAction";
		parms['event'] = "applyByPassword";
		OZONE.ajax.requestModule("membership/MembershipByPasswordResultModule", parms, Wikijump.modules.MembershipByPasswordModule.callbacks.apply);
	}
}

Wikijump.modules.MembershipByPasswordModule.callbacks = {
	apply: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessDialog();
			w.content = "Congratulations! You are now a member of this site!";
			w.addButtonListener('close message', function(){window.location.reload()});
			w.show();
		} else {
			if(!Wikijump.utils.handleError(r)) {return;}
		}
		return;

	}

}

Wikijump.modules.MembershipByPasswordModule.init = function(){
}

Wikijump.modules.MembershipByPasswordModule.init();
