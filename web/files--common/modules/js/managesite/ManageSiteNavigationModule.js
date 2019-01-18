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

WIKIDOT.modules.ManagerSiteNavigationModule = {};

WIKIDOT.modules.ManagerSiteNavigationModule.vars = {
	currentCategory: null
}

WIKIDOT.modules.ManagerSiteNavigationModule.listeners = {
	categoryChange: function(e){
		// update nav info
		var categoryId = document.getElementById("sm-nav-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		WIKIDOT.modules.ManagerSiteNavigationModule.vars.currentCategory = category;
		// check if has a individual nav
		if(category['name'] == "_default"){
			$("sm-nav-noind").style.display = "none";
			$("sm-nav-list").style.display = "";
		} else {
			$("sm-nav-noind").style.display = "block";
			if(category['nav_default'] == true){
				$("sm-nav-noin").checked=true;
				$("sm-nav-list").style.display = "none";
			} else {
				$("sm-nav-noin").checked=false;
				$("sm-nav-list").style.display = "";
			}
		}
		
		WIKIDOT.modules.ManagerSiteNavigationModule.utils.updateNavigationPreview();
	
	},
	
	indClick: function(e){
		var categoryId = document.getElementById("sm-nav-cats").value;
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		if($("sm-nav-noin").checked == true){
			$("sm-nav-list").style.display = "none";
			category['nav_default'] = true;
		}else{
			$("sm-nav-list").style.display = "";
			category['nav_default'] = false;
		}
		WIKIDOT.modules.ManagerSiteNavigationModule.utils.updateNavigationPreview();
	},
	
	navChange: function(e){
		// save changes to the array
		var category = WIKIDOT.modules.ManagerSiteNavigationModule.vars.currentCategory;
		var topBar = document.getElementById("sm-nav-top-bar").value;
		var sideBar =  document.getElementById("sm-nav-side-bar").value;
		category["top_bar_page_name"] = topBar;
		category["side_bar_page_name"] = sideBar;
		WIKIDOT.modules.ManagerSiteNavigationModule.utils.updateNavigationPreview();
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
		parms['event'] = "saveNavigation";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManagerSiteNavigationModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
	
}

WIKIDOT.modules.ManagerSiteNavigationModule.callbacks = {
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

WIKIDOT.modules.ManagerSiteNavigationModule.utils = {
	updateNavigationPreview: function(){
		// apart from just updating the preview also show/hide extra textarea
		// for custom navs
		var category = WIKIDOT.modules.ManagerSiteNavigationModule.vars.currentCategory;
		document.getElementById("sm-nav-top-bar").value = category['top_bar_page_name'];
		document.getElementById("sm-nav-side-bar").value = category['side_bar_page_name'];
	}
}

WIKIDOT.modules.ManagerSiteNavigationModule.init = function(){
	YAHOO.util.Event.addListener("sm-nav-cats", "change", WIKIDOT.modules.ManagerSiteNavigationModule.listeners.categoryChange);
	
	YAHOO.util.Event.addListener("sm-nav-noind", "click", WIKIDOT.modules.ManagerSiteNavigationModule.listeners.indClick);
	
	var ids = ["sm-nav-top-bar", "sm-nav-side-bar"];
	YAHOO.util.Event.addListener(ids, 'keyup',  WIKIDOT.modules.ManagerSiteNavigationModule.listeners.navChange);
	
	YAHOO.util.Event.addListener("sm-nav-cancel", "click", WIKIDOT.modules.ManagerSiteNavigationModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-nav-save", "click", WIKIDOT.modules.ManagerSiteNavigationModule.listeners.save);
	// init categories info
	WIKIDOT.modules.ManagerSiteNavigationModule.listeners.categoryChange(null);
	
	// attach the autocomplete thing
	var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
	myDataSource.scriptQueryParam="q";
	myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";

	var myAutoComp = new YAHOO.widget.AutoComplete("sm-nav-top-bar","sm-nav-top-bar-list", myDataSource);
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
	myAutoComp.useIFrame = true;
	
	var myAutoComp2 = new YAHOO.widget.AutoComplete("sm-nav-side-bar","sm-nav-side-bar-list", myDataSource);
	myAutoComp2.formatResult = function(aResultItem, sQuery) { 
		var title = aResultItem[1];
		var unixName = aResultItem[0];
		if(unixName!= null){
			return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 80%;">('+title+')</div>';
		} else {
			return "";
		}
	}
	
	myAutoComp.autoHighlight = false;
	myAutoComp2.minQueryLength = 2;
	myAutoComp2.queryDelay = 0.5;
}

WIKIDOT.modules.ManagerSiteNavigationModule.init();
