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

WIKIDOT.modules.ManagerSiteGeneralModule = {};

WIKIDOT.modules.ManagerSiteGeneralModule.listeners = {
	
	save: function(e){
		var parms = OZONE.utils.formToArray("sm-general-form");
		parms['action'] = "ManageSiteAction";
		parms['event'] = "saveGeneral";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManagerSiteGeneralModule.callbacks.save);	
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	
	},
	cancel: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	}
}

WIKIDOT.modules.ManagerSiteGeneralModule.callbacks = {
	save: function(r){
		if(r.status=="form_errors"){
			OZONE.dialog.cleanAll();
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";
			
			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}		
			inner += "</ul>";
			$("sm-general-errorblock").innerHTML = inner;
			$("sm-general-errorblock").style.display="block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("sm-general-errorblock").style.display="none";
		var w = new OZONE.dialogs.SuccessBox();
		w.content ="Changes saved.";
		w.show();
	},
	cancel: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("site-manager", r.body);
	}
}

WIKIDOT.modules.ManagerSiteGeneralModule.init = function(){
	YAHOO.util.Event.addListener("sm-general-cancel", "click", WIKIDOT.modules.ManagerSiteGeneralModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-general-save", "click", WIKIDOT.modules.ManagerSiteGeneralModule.listeners.save);
	var limiter = new OZONE.forms.lengthLimiter("site-description-field", "site-description-field-left", 300);

	// attach the autocomplete thing
	var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
	myDataSource.scriptQueryParam="q";
	myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";

	var myAutoComp = new YAHOO.widget.AutoComplete("sm-general-start","sm-general-start-list", myDataSource);
	myAutoComp.formatResult = function(aResultItem, sQuery) { 
		var title = aResultItem[1];
		var unixName = aResultItem[0];
		if(unixName!= null){
			return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 80%;">('+title+')</div>';
		} else {
			return "";
		}
	}
	
	myAutoComp.autoHighlight = false;
	myAutoComp.minQueryLength = 2;
	myAutoComp.queryDelay = 0.5;
		
}

WIKIDOT.modules.ManagerSiteGeneralModule.init();
