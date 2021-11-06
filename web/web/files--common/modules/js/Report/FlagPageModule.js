

Wikijump.modules.FlagPageModule = {};

Wikijump.modules.FlagPageModule.listeners = {
	setFlag: function(e, flag){
		var p = new Object();
		p.action = "AbuseFlagAction";
		p.event = "FlagPage";
		if(flag){
			p.flag = "yes";
			$("flag-page-options-flag").style.display="none";
			$("flag-page-options-unflag").style.display="block";
		}else{
			$("flag-page-options-flag").style.display="block";
			$("flag-page-options-unflag").style.display="none";
		}
		OZONE.ajax.requestModule(null, p, Wikijump.modules.FlagPageModule.callbacks.setFlag);

	}

}

Wikijump.modules.FlagPageModule.callbacks = {
	setFlag: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
	}

}
