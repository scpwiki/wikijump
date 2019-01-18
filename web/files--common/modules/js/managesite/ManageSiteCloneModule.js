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

WIKIDOT.modules.ManageSiteCloneModule = {};

WIKIDOT.modules.ManageSiteCloneModule.vars = {
	unixname: null
}

WIKIDOT.modules.ManageSiteCloneModule.listeners = {
	
	cloneSite: function(e){
		var p = OZONE.utils.formToArray($("clone-site-form"));
		p.action = "ManageSiteCloneAction";
		p.event = "cloneSite";
		OZONE.ajax.requestModule("managesite/ManageSiteClone2Module", p, WIKIDOT.modules.ManageSiteCloneModule.callbacks.cloneSite);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Cloning site...";
		w.show();
		WIKIDOT.modules.ManageSiteCloneModule.vars.unixname = p.unixname;
	
	},
	cancel: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	},
	
	goToTheSite: function(e){
		window.location.href = 'http://'+WIKIDOT.modules.ManageSiteCloneModule.vars.unixname+"."+URL_DOMAIN;
	}
}

WIKIDOT.modules.ManageSiteCloneModule.callbacks = {
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
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("clone-site-form-errors").innerHTML = '';
		$("sm-clone-block").innerHTML = r.body;
		
		OZONE.dialog.cleanAll();
	}
}

WIKIDOT.modules.ManageSiteCloneModule.init = function(){
	
}

WIKIDOT.modules.ManageSiteCloneModule.init();
