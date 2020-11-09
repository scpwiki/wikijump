

Wikijump.modules.ManagerSiteTemplatesModule = {};

Wikijump.modules.ManagerSiteTemplatesModule.vars = {
	currentCategory: null
}

Wikijump.modules.ManagerSiteTemplatesModule.listeners = {
	categoryChange: function(e){
		// update template info
		var categoryId = $("sm-template-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		Wikijump.modules.ManagerSiteTemplatesModule.vars.currentCategory = category;

		var value = category['template_id'];
		if(value == null){
			$("sm-templates-list").value = "";
		} else {
			$("sm-templates-list").value=value;
		}
		Wikijump.modules.ManagerSiteTemplatesModule.utils.updateTemplatePreview();

	},

	templateChange: function(e){
		// save changes to the array
		var categoryId = $("sm-template-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		if(this.value == ""){
			category['template_id'] = null;
		} else {
			category['template_id'] = this.value;
		}
		Wikijump.modules.ManagerSiteTemplatesModule.utils.updateTemplatePreview();
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
		parms['event'] = "saveTemplates";
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManagerSiteTemplatesModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}

}

Wikijump.modules.ManagerSiteTemplatesModule.callbacks = {
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	},

	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content ="Changes saved.";
		w.show();
	}

}

Wikijump.modules.ManagerSiteTemplatesModule.utils = {
	updateTemplatePreview: function(){
		// apart from just updating the preview also show/hide extra textarea
		// for custom licenses
		var category = Wikijump.modules.ManagerSiteTemplatesModule.vars.currentCategory;
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

Wikijump.modules.ManagerSiteTemplatesModule.init = function(){
	YAHOO.util.Event.addListener("sm-template-cats", "change", Wikijump.modules.ManagerSiteTemplatesModule.listeners.categoryChange);
	YAHOO.util.Event.addListener("sm-templates-list", "change", Wikijump.modules.ManagerSiteTemplatesModule.listeners.templateChange);

	YAHOO.util.Event.addListener("sm-templates-cancel", "click", Wikijump.modules.ManagerSiteTemplatesModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-templates-save", "click", Wikijump.modules.ManagerSiteTemplatesModule.listeners.save);
	// init categories info
	if($("sm-template-cats")){
		Wikijump.modules.ManagerSiteTemplatesModule.listeners.categoryChange(null);
	}
}

Wikijump.modules.ManagerSiteTemplatesModule.init();
