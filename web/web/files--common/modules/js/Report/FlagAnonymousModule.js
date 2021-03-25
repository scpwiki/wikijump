

Wikijump.modules.FlagAnonymousModule = {};

Wikijump.modules.FlagAnonymousModule.listeners = {
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
		OZONE.ajax.requestModule(null, p, Wikijump.modules.FlagAnonymousModule.callbacks.setFlag);

	}

}

Wikijump.modules.FlagAnonymousModule.callbacks = {
	setFlag: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
	}

}
