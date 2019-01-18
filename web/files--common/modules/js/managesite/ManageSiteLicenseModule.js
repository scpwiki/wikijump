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

WIKIDOT.modules.ManagerSiteLicenseModule = {};

WIKIDOT.modules.ManagerSiteLicenseModule.vars = {
	currentCategory: null,
	limiter: null
}

WIKIDOT.modules.ManagerSiteLicenseModule.listeners = {
	categoryChange: function(e){
		// update license info
		var categoryId = document.getElementById("sm-license-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		WIKIDOT.modules.ManagerSiteLicenseModule.vars.currentCategory = category;
		// check if has a individual license
		if(category['name'] == "_default"){
			document.getElementById("sm-license-noind").style.display = "none";
			document.getElementById("sm-license-list").style.display = "";
		} else {
			document.getElementById("sm-license-noind").style.display = "block";
			if(category['license_default'] == true){
				document.getElementById("sm-license-noin").checked=true;
				document.getElementById("sm-license-list").style.display = "none";
			} else {
				document.getElementById("sm-license-noin").checked=false;
				document.getElementById("sm-license-list").style.display = "";
			}
		}
		
		$("sm-license-lic").value=category['license_id'];
		WIKIDOT.modules.ManagerSiteLicenseModule.utils.updateLicensePreview();
	
	},
	
	indClick: function(e){
		var categoryId = $("sm-license-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		if($("sm-license-noin").checked == true){
			$("sm-license-list").style.display = "none";
			category['license_default'] = true;
		}else{
			$("sm-license-list").style.display = "";
			category['license_default'] = false;
		}
		WIKIDOT.modules.ManagerSiteLicenseModule.utils.updateLicensePreview();
	},
	
	licenseChange: function(e){
		// save changes to the array
		var categoryId = document.getElementById("sm-license-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		category['license_id'] = this.value;
		WIKIDOT.modules.ManagerSiteLicenseModule.utils.updateLicensePreview();
	},
	
	otherDescriptionChange: function(e){
		var category = WIKIDOT.modules.ManagerSiteLicenseModule.vars.currentCategory;
		var text = $("sm-other-license-text").value;
		category['license_other'] = text;
		
		// also update the preview...
		var licenseId = document.getElementById("sm-license-lic").value;
		var lid = "sm-prev-license-"+licenseId;
		var prev = $(lid);
		text = text.split("&").join("&amp;").split("<").join("&lt;").split(">").join("&gt;");
		// now reenable some tags, i.e.: "a", "img" and "br"
		text = text.replace(/&lt;a href="(.*?)"&gt;(.*?)&lt;\/a&gt;/g, '<a href="$1">$2</a>') ;
		text = text.replace(/&lt;img src="(.*?)"(?: alt="(.*?)")?(?: )*(?:\/)?&gt;/g, '<img src="$1" alt="$2"/>');
		text = text.replace(/&lt;br(\/)?&gt;/g, '<br/>');
		text = text.replace(/&lt;strong&gt;(.*?)&lt;\/strong&gt;/g, '<strong>$1</strong>');
		text = text.replace(/&lt;em&gt;(.*?)&lt;\/em&gt;/g, '<em>$1</em>');
		
		prev.innerHTML = text;
		
	},
	
	cancel: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	},
	
	save: function(e){
		// ok, do it the easy way: serialize categories using the JSON method
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		var parms = new Object();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteAction";
		parms['event'] = "saveLicense";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManagerSiteLicenseModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
	
}

WIKIDOT.modules.ManagerSiteLicenseModule.callbacks = {
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	},
	
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes have been saved";
		w.show();
	}
	
}

WIKIDOT.modules.ManagerSiteLicenseModule.utils = {
	updateLicensePreview: function(){
		// apart from just updating the preview also show/hide extra textarea
		// for custom licenses
		var licenseId;
		var category = WIKIDOT.modules.ManagerSiteLicenseModule.vars.currentCategory;
		if($("sm-license-noin").checked == true && category['name'] != "_default"){
			// get theme_id for the category _default
			var defCategory = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryByName("_default");
			licenseId = defCategory['license_id'];
		} else {
			licenseId = $("sm-license-lic").value;
		}

		// let us assume that "other" has id = 1. bleeeeh
		
		if(licenseId == 1){
			$("sm-other-license").style.display="block";	
			// fill it with contents
			if(category['name'] == '_default' || $("sm-license-noin").checked == false){
				$("sm-other-license-text").value = category['license_other'];
			}else{
				var defCategory = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryByName("_default");
				$("sm-other-license-text").value =  defCategory['license_other'];
			}
			WIKIDOT.modules.ManagerSiteLicenseModule.listeners.otherDescriptionChange();
			WIKIDOT.modules.ManagerSiteLicenseModule.vars.limiter.keyListener(null);
		} else {
			$("sm-other-license").style.display="none";
		}
		
		// now enable or disable preview
		// first hide all previews
		var div = $("sm-license-preview");
		var pres = div.getElementsByTagName("div");
		for(var i = 0; i< pres.length; i++){
			pres[i].style.display = "none";
		}
		// now show the chosen one
		var pre = $("sm-prev-license-"+licenseId);
		pre.style.display = "block";
		
		return;
		var categoryId = $("sm-license-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		
		// get current license_id
		if($("sm-license-noin").checked == true && category['name'] != "_default"){
			// get license_id for the category _default
			defCategory = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryByName("_default");
			licenseId = defCategory['license_id'];
		} else {
			licenseId = $("sm-license-lic").value;
		}
		OZONE.utils.setInnerHTMLContent("sm-license-preview", "preview of license: "+licenseId);
	}
}

WIKIDOT.modules.ManagerSiteLicenseModule.init = function(){
	
	YAHOO.util.Event.addListener("sm-license-cats", "change", WIKIDOT.modules.ManagerSiteLicenseModule.listeners.categoryChange);
	YAHOO.util.Event.addListener("sm-license-lic", "change", WIKIDOT.modules.ManagerSiteLicenseModule.listeners.licenseChange);
	YAHOO.util.Event.addListener("sm-license-noind", "click", WIKIDOT.modules.ManagerSiteLicenseModule.listeners.indClick);
	YAHOO.util.Event.addListener("sm-other-license-text", "keyup", WIKIDOT.modules.ManagerSiteLicenseModule.listeners.otherDescriptionChange);
	
	YAHOO.util.Event.addListener("sm-license-cancel", "click", WIKIDOT.modules.ManagerSiteLicenseModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-license-save", "click", WIKIDOT.modules.ManagerSiteLicenseModule.listeners.save);
	// init categories info

	var limiter = new OZONE.forms.lengthLimiter("sm-other-license-text", "sm-other-license-text-left", 300);
	WIKIDOT.modules.ManagerSiteLicenseModule.vars.limiter = limiter;
	
	WIKIDOT.modules.ManagerSiteLicenseModule.listeners.categoryChange(null);
	
}

WIKIDOT.modules.ManagerSiteLicenseModule.init();
