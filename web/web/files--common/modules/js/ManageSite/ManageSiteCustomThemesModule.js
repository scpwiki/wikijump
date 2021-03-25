

Wikijump.modules.ManageSiteCustomThemesModule = {};

Wikijump.modules.ManageSiteCustomThemesModule.vars = {
	currentThemeId1: null,
	editThemeId: null
}

Wikijump.modules.ManageSiteCustomThemesModule.listeners = {
	editTheme: function(e, themeId){
		var p = new Object();
		if(themeId){
			p['themeId'] = themeId;
		}
		Wikijump.modules.ManageSiteCustomThemesModule.vars.editThemeId = themeId;
		OZONE.ajax.requestModule("ManageSite/ManageSiteEditCustomThemeModule", p, Wikijump.modules.ManageSiteCustomThemesModule.callbacks.editTheme);

	},

	importCss: function(e){
		var p = new Object();
		p.pageName = $("sm-cssimport-input").value;
		if(p.pageName == ''){
			$("cssimport-error").innerHTML = "In order to import CSS you should first " +
					"give a non-empty page name.";
			$("cssimport-error").style.display = "block";
			return;
		}
		p.action = "ManageSiteAction";
		p.event = "importCss";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteCustomThemesModule.callbacks.importCss);

	},

	cancelEditTheme: function(e){
		$("edit-theme-box").innerHTML = "";
	},

	saveTheme: function(e){
		var p = OZONE.utils.formToArray("sm-edit-theme-form");
		p.action = "ManageSiteAction";
		p.event = "customThemeSave";
		if(Wikijump.modules.ManageSiteCustomThemesModule.vars.editThemeId){
			p.themeId = Wikijump.modules.ManageSiteCustomThemesModule.vars.editThemeId;
		}
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteCustomThemesModule.callbacks.saveTheme);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving theme...";
		w.show();
	},
	deleteTheme: function(e, themeId){
		Wikijump.modules.ManageSiteCustomThemesModule.vars.currentThemeId1 = themeId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = "Are you sure you want to delete this theme?";
		w.buttons = ["cancel", "yes, delete"];
		w.addButtonListener("cancel", w.close);
		w.addButtonListener("yes, delete", function(e){
			Wikijump.modules.ManageSiteCustomThemesModule.listeners.deleteTheme2(e, Wikijump.modules.ManageSiteCustomThemesModule.vars.currentThemeId1);
		});
		w.show();
	},
	deleteTheme2: function(e, themeId){
		var p = new Object();
		if(themeId){p['themeId'] = themeId;}
		p.action = "ManageSiteAction";
		p.event = "customThemeDelete";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteCustomThemesModule.callbacks.deleteTheme);
	}
}

Wikijump.modules.ManageSiteCustomThemesModule.callbacks = {
	editTheme: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("edit-theme-box").innerHTML = r.body;

		// attach autocomplete
		// attach the autocomplete thing
		var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']);
		myDataSource.scriptQueryParam="q";
		myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";

		var myAutoComp = new YAHOO.widget.AutoComplete("sm-cssimport-input","sm-cssimport-input-list", myDataSource);
		myAutoComp.formatResult = function(aResultItem, sQuery) {
			var title = aResultItem[1];
			var unixName = aResultItem[0];
			if(unixName!= null){
				return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 80%;">('+title+')</div>';
			} else {
				return "";
			}
		}
		myAutoComp.minQueryLength = 2;
		myAutoComp.queryDelay = 0.5;
	},

	importCss: function(r){
		if(r.status == "form_error"){
			$("cssimport-error").innerHTML = r.message;
			$("cssimport-error").style.display = "block";
			return;
		}
		$("cssimport-error").style.display = "none";
		if(!Wikijump.utils.handleError(r)) {return;}
		$("sm-csscode").value=r.code;
	},
	saveTheme: function(r){
		if(r.status == "form_error"){
			$("edit-theme-error").innerHTML = r.message;
			$("edit-theme-error").style.display = "block";
			OZONE.visuals.scrollTo($("edit-theme-error"));
			OZONE.dialog.cleanAll();
			return;
		}
		$("edit-theme-error").style.display = "none";
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Theme saved.";
		w.show();
		setTimeout('Wikijump.modules.ManagerSiteModule.utils.loadModule("sm-customthemes")', 1000);

	},

	deleteTheme: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		Wikijump.modules.ManagerSiteModule.utils.loadModule("sm-customthemes");
		OZONE.dialog.cleanAll();
	}
}
