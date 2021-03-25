

Wikijump.modules.ManageSiteCloneModule = {};

Wikijump.modules.ManageSiteCloneModule.vars = {
	unixname: null
}

Wikijump.modules.ManageSiteCloneModule.listeners = {

	cloneSite: function(e){
		var p = OZONE.utils.formToArray($("clone-site-form"));
		p.action = "ManageSiteCloneAction";
		p.event = "cloneSite";
		OZONE.ajax.requestModule("ManageSite/ManageSiteClone2Module", p, Wikijump.modules.ManageSiteCloneModule.callbacks.cloneSite);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Cloning site...";
		w.show();
		Wikijump.modules.ManageSiteCloneModule.vars.unixname = p.unixname;

	},
	cancel: function(e){
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	},

	goToTheSite: function(e){
		window.location.href = HTTP_SCHEMA+"://"+Wikijump.modules.ManageSiteCloneModule.vars.unixname+"."+URL_DOMAIN;
	}
}

Wikijump.modules.ManageSiteCloneModule.callbacks = {
	cloneSite: function(r){
		if(r.status=="form_errors"){
			OZONE.dialog.cleanAll();
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";

			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}
			inner += "</ul>";
			$("clone-site-form-errors").innerHTML = inner;
			$("clone-site-form-errors").style.display="block";
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}
		$("clone-site-form-errors").innerHTML = '';
		$("sm-clone-block").innerHTML = r.body;

		OZONE.dialog.cleanAll();
	}
}

Wikijump.modules.ManageSiteCloneModule.init = function(){

}

Wikijump.modules.ManageSiteCloneModule.init();
