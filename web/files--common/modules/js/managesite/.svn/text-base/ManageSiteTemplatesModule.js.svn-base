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

WIKIDOT.modules.ManagerSiteTemplatesModule = {};

WIKIDOT.modules.ManagerSiteTemplatesModule.vars = {
	currentCategory: null
}

WIKIDOT.modules.ManagerSiteTemplatesModule.listeners = {
	categoryChange: function(e){
		// update template info
		var categoryId = $("sm-template-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		WIKIDOT.modules.ManagerSiteTemplatesModule.vars.currentCategory = category;
		
		var value = category['template_id'];
		if(value == null){
			$("sm-templates-list").value = "";
		} else {
			$("sm-templates-list").value=value;
		}
		WIKIDOT.modules.ManagerSiteTemplatesModule.utils.updateTemplatePreview();
	
	},
	
	templateChange: function(e){
		// save changes to the array
		var categoryId = $("sm-template-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		if(this.value == ""){
			category['template_id'] = null;
		} else {
			category['template_id'] = this.value;
		}
		WIKIDOT.modules.ManagerSiteTemplatesModule.utils.updateTemplatePreview();
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
		parms['event'] = "saveTemplates";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManagerSiteTemplatesModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
	
}

WIKIDOT.modules.ManagerSiteTemplatesModule.callbacks = {
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	},
	
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content ="Changes saved.";
		w.show();
	}
	
}

WIKIDOT.modules.ManagerSiteTemplatesModule.utils = {
	updateTemplatePreview: function(){
		// apart from just updating the preview also show/hide extra textarea
		// for custom licenses
		var category = WIKIDOT.modules.ManagerSiteTemplatesModule.vars.currentCategory;
		var templateId = $("sm-templates-list").value;
		// let us assume that "other" has id = 11. bleeeeh

		// now enable or disable preview
		// first hide all previews

		var div = $("sm-template-preview");
		
		if(templateId==""){
			div.style.display = "none";
			return;
		} else {
			div.style.display = "block";
		}
		var pres = div.getElementsByTagName("div");
		for(var i = 0; i< pres.length; i++){
			pres[i].style.display = "none";
		}
		// now show the chosen one
		var pre = $("sm-template-preview-"+templateId);
		pre.style.display = "block";
		
		return;
	}
}

WIKIDOT.modules.ManagerSiteTemplatesModule.init = function(){
	YAHOO.util.Event.addListener("sm-template-cats", "change", WIKIDOT.modules.ManagerSiteTemplatesModule.listeners.categoryChange);
	YAHOO.util.Event.addListener("sm-templates-list", "change", WIKIDOT.modules.ManagerSiteTemplatesModule.listeners.templateChange);
	
	YAHOO.util.Event.addListener("sm-templates-cancel", "click", WIKIDOT.modules.ManagerSiteTemplatesModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-templates-save", "click", WIKIDOT.modules.ManagerSiteTemplatesModule.listeners.save);
	// init categories info
	if($("sm-template-cats")){
		WIKIDOT.modules.ManagerSiteTemplatesModule.listeners.categoryChange(null);
	}
}

WIKIDOT.modules.ManagerSiteTemplatesModule.init();
