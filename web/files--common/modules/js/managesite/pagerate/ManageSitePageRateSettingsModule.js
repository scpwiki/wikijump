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

WIKIDOT.modules.ManageSitePageRateSettingsModule = {};

WIKIDOT.modules.ManageSitePageRateSettingsModule.listeners ={
	save: function(e){
		WIKIDOT.modules.ManageSitePageRateSettingsModule.utils.updateFromForm();
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		var parms = new Object();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteAction";
		parms['event'] = "savePageRateSettings";
		
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManageSitePageRateSettingsModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
};

WIKIDOT.modules.ManageSitePageRateSettingsModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
};

WIKIDOT.modules.ManageSitePageRateSettingsModule.utils = {
	updateFromForm: function(){
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		var id;
		for(var i=0; i<categories.length; i++){
			// check for the value in the form
			id = "cat235-"+categories[i].category_id;
			var ps = '';
			if($(id+"-e").value=='enabled'){ 
				ps += 'e';
			}else if($(id+"-e").value=='disabled'){
				ps += 'd';
			}
			
			if($(id+"-w").value=='r'){ 
				ps += 'r';
			}else if($(id+"-w").value=='m'){
				ps += 'm';
			}
			
			if($(id+"-v").value=='v'){ 
				ps += 'v';
			}else if($(id+"-v").value=='a'){
				ps += 'a';
			}
			
			ps += $(id+"-t").value;

			categories[i].rating = ps;
		}
	},
	
	updateVis: function(categoryId){
		var id = "cat235-"+categoryId;
		if($(id+"-e").value=='enabled'){ 
			$(id+"-w").style.visibility="visible";
			$(id+"-v").style.visibility="visible";
			$(id+"-t").style.visibility="visible";
		}else{
			$(id+"-w").style.visibility="hidden";
			$(id+"-v").style.visibility="hidden";
			$(id+"-t").style.visibility="hidden";
		}
	}
	
};

WIKIDOT.modules.ManageSitePageRateSettingsModule.init = function(){
	var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
	for(var i=0; i<categories.length; i++){
		WIKIDOT.modules.ManageSitePageRateSettingsModule.utils.updateVis(categories[i].category_id);
	}
}

WIKIDOT.modules.ManageSitePageRateSettingsModule.init();
