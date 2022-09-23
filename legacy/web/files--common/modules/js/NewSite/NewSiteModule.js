Wikijump.modules.NewSiteModule = {};

Wikijump.modules.NewSiteModule.listeners = {
	next1: function(e){
		OZONE.ajax.requestModule("NewSite/NewSite1Module", null, Wikijump.modules.NewSiteModule.callbacks.next1);
	},

	next2: function(e){
		OZONE.ajax.requestModule("NewSite/NewSite2Module", null, Wikijump.modules.NewSiteModule.callbacks.next2);
	},

	next3: function(e){
		var p = OZONE.utils.formToArray($("new-site-form"));
		p.action = "NewSiteAction";
		p.event = "createSite";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.NewSiteModule.callbacks.next3);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Creating site...";
		w.show();
	}

}

Wikijump.modules.NewSiteModule.callbacks = {
	next1: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("new-site-box").innerHTML = r.body;
	},
	next2: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("new-site-box").innerHTML = r.body;
		var limiter = new OZONE.forms.lengthLimiter("site-description-field", "site-description-field-left", 300);

	},
	next3: function(r){
		if(r.status=="form_errors"){
			OZONE.dialog.cleanAll();
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";

			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}
			inner += "</ul>";
			$("new-site-form-errors").innerHTML = inner;
			$("new-site-form-errors").style.display="block";
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "New site successfuly created!";
		w.show();

		setTimeout("window.location.href='"+HTTP_SCHEMA+"://"+r.siteUnixName+"."+URL_DOMAIN+"'", 1000);;
	}
}
