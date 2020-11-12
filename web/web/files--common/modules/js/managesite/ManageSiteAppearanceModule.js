

Wikijump.modules.ManagerSiteAppearanceModule = {};

Wikijump.modules.ManagerSiteAppearanceModule.vars = {

}

Wikijump.modules.ManagerSiteAppearanceModule.listeners = {
	categoryChange: function(e){
		// update theme info
		var categoryId = $("sm-appearance-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		// check if has a individual theme
		Wikijump.modules.ManagerSiteAppearanceModule.utils.hideVariants();
		if(category['name'] == "_default"){
			$("sm-appearance-noind").style.display = "none";
			$("sm-appearance-theme").style.display = "block";
			var ez = $("sm-appearance-variants-"+category['theme_id']);
			if(ez){
				ez.style.display = "block";
				if(category['variant_theme_id']){
					$("sm-appearance-variants-select-"+category['theme_id']).value=category['variant_theme_id'];
				}else{
					$("sm-appearance-variants-select-"+category['theme_id']).value=category['theme_id'];
				}
			}else{
				category['variant_theme_id'] = null;
			}
		} else {
			$("sm-appearance-noind").style.display = "block";
			if(category['theme_default'] == true){
				$("sm-appearance-noin").checked=true;
				$("sm-appearance-theme").style.display = "none";
			} else {
				$("sm-appearance-noin").checked=false;
				$("sm-appearance-theme").style.display = "block";
				var ez = $("sm-appearance-variants-"+category['theme_id']);
				if(ez){
					ez.style.display = "block";
					if(category['variant_theme_id']){
						$("sm-appearance-variants-select-"+category['theme_id']).value=category['variant_theme_id'];
					}else{
						$("sm-appearance-variants-select-"+category['theme_id']).value=category['theme_id'];
					}
				}else{
					category['variant_theme_id'] = null;
				}
			}
		}
		//if(category['theme_external_url']){
			$('sm-appearance-external-url').value = category['theme_external_url'];
		//}

		$("sm-appearance-theme-id").value=category['theme_id'];
		Wikijump.modules.ManagerSiteAppearanceModule.utils.updateThemePreview();

	},

	indClick: function(e){
		var categoryId = $("sm-appearance-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		if($("sm-appearance-noin").checked == true){
			$("sm-appearance-theme").style.display = "none";
			category['theme_default'] = true;
		}else{
			$("sm-appearance-theme").style.display = "";
			category['theme_default'] = false;

			var ez = $("sm-appearance-variants-"+category['theme_id']);
				if(ez){
					ez.style.display = "block";
					if(category['variant_theme_id']){
						$("sm-appearance-variants-select-"+category['theme_id']).value=category['variant_theme_id'];
					}else{
						$("sm-appearance-variants-select-"+category['theme_id']).value=category['theme_id'];
					}
				}

		}
		Wikijump.modules.ManagerSiteAppearanceModule.utils.updateThemePreview();
	},

	themeChange: function(e){
		// save changes to the array
		var categoryId = $("sm-appearance-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		if(this.tagName.toLowerCase() == 'select'){
			category['theme_id'] = this.value;
		}
		Wikijump.modules.ManagerSiteAppearanceModule.utils.hideVariants();
		var ez = $("sm-appearance-variants-"+$("sm-appearance-theme-id").value);
		category['variant_theme_id'] = null;

		if(ez){
			//alert('variant');
			ez.style.display = "block";
			if(category['variant_theme_id']){
				$("sm-appearance-variants-select-"+category['theme_id']).value=category['variant_theme_id'];
			}else if($("sm-appearance-variants-select-"+category['theme_id'])){
				$("sm-appearance-variants-select-"+category['theme_id']).value=category['theme_id'];
			}
		}else{
			category['variant_theme_id'] = null;
		}

		/* Handle external themes. */
		var exurl = $('sm-appearance-external-url').value;
		//if(exurl != '' && exurl.match('^https?://')){
			category['theme_external_url'] = exurl;
		//}
		Wikijump.modules.ManagerSiteAppearanceModule.utils.updateThemePreview();
	},

	variantChange: function(e){
		var categoryId = $("sm-appearance-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		var themeId = $("sm-appearance-theme-id").value;
		var variantThemeId = $("sm-appearance-variants-select-"+themeId).value;
		category['variant_theme_id'] = variantThemeId;
	},

	cancel: function(e){
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	},

	save: function(e){
		// ok, do it the easy way: serialize categories using the JSON method
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		//alert(serialized);
		var parms = new Object();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteAction";
		parms['event'] = "saveAppearance";
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManagerSiteAppearanceModule.callbacks.save);
	}

}

Wikijump.modules.ManagerSiteAppearanceModule.callbacks = {
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

Wikijump.modules.ManagerSiteAppearanceModule.utils = {
	updateThemePreview: function(){
		var categoryId = $("sm-appearance-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		var themeId;
		// get current theme_id
		if($("sm-appearance-noin").checked == true && category['name'] != "_default"){
			// get theme_id for the category _default
			var defCategory = Wikijump.modules.ManagerSiteModule.utils.getCategoryByName("_default");
			themeId = defCategory['theme_id'];
		} else {
			themeId = $("sm-appearance-theme-id").value;
		}

		// hide all previews first
		var prs = $("sm-appearance-theme-preview").childNodes;
		for(var i=0; i<prs.length; i++){
			if(prs.tagName=='div') {prs[i].style.display="none";}
		}
		var previewDiv = $("sm-theme-preview-"+themeId);
		if(previewDiv){
			previewDiv.style.display="block";
			$("sm-appearance-theme-preview").style.display="block";
		}else{
			$("sm-appearance-theme-preview").style.display="none";
		}

	},
	hideVariants: function(){
		var divs = $("theme-variants-container").getElementsByTagName("div");
		for(var i=0; i<divs.length; i++){
			divs[i].style.display="none";
		}
	}
}

Wikijump.modules.ManagerSiteAppearanceModule.init = function(){
	YAHOO.util.Event.addListener("sm-appearance-cats", "change", Wikijump.modules.ManagerSiteAppearanceModule.listeners.categoryChange);
	YAHOO.util.Event.addListener("sm-appearance-theme-id", "change", Wikijump.modules.ManagerSiteAppearanceModule.listeners.themeChange);
	YAHOO.util.Event.addListener("sm-appearance-external-url", "change", Wikijump.modules.ManagerSiteAppearanceModule.listeners.themeChange);
	YAHOO.util.Event.addListener("sm-appearance-noind", "click", Wikijump.modules.ManagerSiteAppearanceModule.listeners.indClick);

	YAHOO.util.Event.addListener("sm-appearance-cancel", "click", Wikijump.modules.ManagerSiteAppearanceModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-appearance-save", "click", Wikijump.modules.ManagerSiteAppearanceModule.listeners.save);

	// init categories info
	Wikijump.modules.ManagerSiteAppearanceModule.listeners.categoryChange(null);
}

Wikijump.modules.ManagerSiteAppearanceModule.init();
