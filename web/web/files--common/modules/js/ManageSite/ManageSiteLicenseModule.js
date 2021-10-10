Wikijump.modules.ManagerSiteLicenseModule = {};

Wikijump.modules.ManagerSiteLicenseModule.vars = {
	currentCategory: null,
	limiter: null
}

Wikijump.modules.ManagerSiteLicenseModule.listeners = {
	categoryChange: function(e){
		// update license info
		var categoryId = document.getElementById("sm-license-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		Wikijump.modules.ManagerSiteLicenseModule.vars.currentCategory = category;
		// check if has a individual license
		if(category['name'] == "_default"){
			document.getElementById("sm-license-noind").style.display = "none";
			document.getElementById("sm-license-list").style.display = "";
		} else {
			document.getElementById("sm-license-noind").style.display = "block";
			if(category['license_inherits'] == true){
				document.getElementById("sm-license-noin").checked=true;
				document.getElementById("sm-license-list").style.display = "none";
			} else {
				document.getElementById("sm-license-noin").checked=false;
				document.getElementById("sm-license-list").style.display = "";
			}
		}

		$("sm-license-lic").value=category['license_id'];
		Wikijump.modules.ManagerSiteLicenseModule.utils.updateLicensePreview();

	},

	indClick: function(e){
		var categoryId = $("sm-license-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		if($("sm-license-noin").checked == true){
			$("sm-license-list").style.display = "none";
			category['license_inherits'] = true;
		}else{
			$("sm-license-list").style.display = "";
			category['license_inherits'] = false;
		}
		Wikijump.modules.ManagerSiteLicenseModule.utils.updateLicensePreview();
	},

	licenseChange: function(e){
		// save changes to the array
		var categoryId = document.getElementById("sm-license-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		category['license_id'] = this.value;
		Wikijump.modules.ManagerSiteLicenseModule.utils.updateLicensePreview();
	},

	cancel: function(e){
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	},

	save: function(e){
		// ok, do it the easy way: serialize categories using the JSON method
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		var parms = new Object();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteAction";
		parms['event'] = "saveLicense";
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManagerSiteLicenseModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}

}

Wikijump.modules.ManagerSiteLicenseModule.callbacks = {
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	},

	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes have been saved";
		w.show();
	}

}

Wikijump.modules.ManagerSiteLicenseModule.utils = {
	updateLicensePreview: function(){
		// apart from just updating the preview also show/hide extra textarea
		// for custom licenses
		var licenseId;
		var category = Wikijump.modules.ManagerSiteLicenseModule.vars.currentCategory;
		if($("sm-license-noin").checked == true && category['name'] != "_default"){
			// get theme_id for the category _default
			var defCategory = Wikijump.modules.ManagerSiteModule.utils.getCategoryByName("_default");
			licenseId = defCategory['license_id'];
		} else {
			licenseId = $("sm-license-lic").value;
		}

		// now enable or disable preview
		// first hide all previews
		var div = $("sm-license-preview");
		var pres = div.getElementsByTagName("div");
		for(var i = 0; i< pres.length; i++){
			pres[i].style.display = "none";
		}
		// now show the chosen one
		var pre = $("sm-prev-license-" + licenseId.replaceAll('_', '-'));
		pre.style.display = "block";

		return;
		var categoryId = $("sm-license-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		// get current license_id
		if($("sm-license-noin").checked == true && category['name'] != "_default"){
			// get license_id for the category _default
			defCategory = Wikijump.modules.ManagerSiteModule.utils.getCategoryByName("_default");
			licenseId = defCategory['license_id'];
		} else {
			licenseId = $("sm-license-lic").value;
		}
		OZONE.utils.setInnerHTMLContent("sm-license-preview", "preview of license: "+licenseId);
	}
}

Wikijump.modules.ManagerSiteLicenseModule.init = function(){

	YAHOO.util.Event.addListener("sm-license-cats", "change", Wikijump.modules.ManagerSiteLicenseModule.listeners.categoryChange);
	YAHOO.util.Event.addListener("sm-license-lic", "change", Wikijump.modules.ManagerSiteLicenseModule.listeners.licenseChange);
	YAHOO.util.Event.addListener("sm-license-noind", "click", Wikijump.modules.ManagerSiteLicenseModule.listeners.indClick);

	YAHOO.util.Event.addListener("sm-license-cancel", "click", Wikijump.modules.ManagerSiteLicenseModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-license-save", "click", Wikijump.modules.ManagerSiteLicenseModule.listeners.save);
	// init categories info

	Wikijump.modules.ManagerSiteLicenseModule.listeners.categoryChange(null);
}

Wikijump.modules.ManagerSiteLicenseModule.init();
