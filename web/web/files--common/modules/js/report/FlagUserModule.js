

Wikijump.modules.FlagUserModule = {};

Wikijump.modules.FlagUserModule.listeners = {
	setFlag: function(e, userId, flag){
		var p = new Object();
		p.path = window.location.pathname;
		p.action = "AbuseFlagAction";
		p.event = "flagUser";
		p.targetUserId = userId;
		if(flag){
			p.flag = "yes";

			if(window.USERINFO && USERINFO.referer){
				p.host = USERINFO.referer;
			}

			$("flag-user-options-flag").style.display="none";
			$("flag-user-options-unflag").style.display="block";
		}else{
			$("flag-user-options-flag").style.display="block";
			$("flag-user-options-unflag").style.display="none";
		}
		OZONE.ajax.requestModule(null, p, Wikijump.modules.FlagUserModule.callbacks.setFlag);

	}

}

Wikijump.modules.FlagUserModule.callbacks = {
	setFlag: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
	}

}
