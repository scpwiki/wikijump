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

WIKIDOT.modules.ManagerSiteDomainModule = {};

WIKIDOT.modules.ManagerSiteDomainModule.listeners = {
	
	save: function(e){
		var domain = $("sm-domain-field").value;
		
		var redirects = new Array();
		var container = $("sm-redirects-box");
		// count them!
		var inputs = container.getElementsByTagName('input');
		for(var i = 0; i<inputs.length; i++){
			var redirUrl = inputs[i].value;
			if(redirUrl){
				if(!redirUrl.match(/^[a-z0-9\-]+(\.[a-z0-9\-]+)+$/i)){
					$("sm-domain-error").innerHTML = '"'+redirUrl+'" is not a valid domain. Please correct it and save again.';
					$("sm-domain-error").style.display = "block";
					return;
				}
			}
			redirects.push(redirUrl);
		}
		
		var redirectsString = redirects.join(';');

		var p = new Object();
		p.redirects = redirectsString;
		p.domain = domain;
		p['action'] = "ManageSiteAction";
		p['event'] = "saveDomain";
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.ManagerSiteDomainModule.callbacks.save);	
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
		YAHOO.util.Event.preventDefault(e);
	
	},
	cancel: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome');	
	},
	clear: function(e){
		$("sm-domain-field").value="";
		$("sm-redirects-box").innerHTML = '';
		WIKIDOT.modules.ManagerSiteDomainModule.listeners.addRedirect(e);
		
	},
	
	addRedirect: function(e){
		var container = $("sm-redirects-box");
		// count them!
		var divs = container.getElementsByTagName('div');
		if(divs.length >= 10){
			alert("Sorry, you can have only up to 10 redirects defined");
			return;
		}
		var inn = $("sm-redirect-template").innerHTML;
		var div = document.createElement('div');
		div.innerHTML = inn;
		container.appendChild(div);
		
	},
	/**
	 * 
	 * @param {Event} e
	 */
	removeRedirect: function(e){
		var el = YAHOO.util.Event.getTarget(e);
		
		var div = null;
		while(el && el.tagName && el.tagName.toLowerCase() != 'div'){
			el = el.parentNode;
		}
		el.parentNode.removeChild(el);

	}
}

WIKIDOT.modules.ManagerSiteDomainModule.callbacks = {
	save: function(r){
		if(r.status=="form_errors"){
			$("sm-domain-errorblock").innerHTML = r.message;
			$("sm-domain-errorblock").style.display="block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content ="Changes saved.";
		w.show();
	},
	cancel: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("site-manager", r.body);
	}
}

WIKIDOT.modules.ManagerSiteDomainModule.init = function(){
	YAHOO.util.Event.addListener("sm-domain-cancel", "click", WIKIDOT.modules.ManagerSiteDomainModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-domain-clear", "click", WIKIDOT.modules.ManagerSiteDomainModule.listeners.clear);
	YAHOO.util.Event.addListener("sm-domain-save", "click", WIKIDOT.modules.ManagerSiteDomainModule.listeners.save);
	YAHOO.util.Event.addListener("sm-domain-form", "submit", WIKIDOT.modules.ManagerSiteDomainModule.listeners.save);
	
}

WIKIDOT.modules.ManagerSiteDomainModule.init();
