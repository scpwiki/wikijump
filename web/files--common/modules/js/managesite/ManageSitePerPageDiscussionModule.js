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

WIKIDOT.modules.ManageSitePerPageDiscussionModule = {};

WIKIDOT.modules.ManageSitePerPageDiscussionModule.listeners ={
	save: function(e){
		WIKIDOT.modules.ManageSitePerPageDiscussionModule.utils.updateFromForm();
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		var parms = new Object();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteForumAction";
		parms['event'] = "savePerPageDiscussion";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManageSitePerPageDiscussionModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
};

WIKIDOT.modules.ManageSitePerPageDiscussionModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
};

WIKIDOT.modules.ManageSitePerPageDiscussionModule.utils = {
	updateFromForm: function(){
		var categories = WIKIDOT.modules.ManagerSiteModule.vars.categories;
		var id;
		for(var i=0; i<categories.length; i++){
			// check for the value in the form
			id = "cat234-"+categories[i].category_id;
			if($(id+"-e").checked){ 
				categories[i].per_page_discussion = true;
			}else if($(id+"-d").checked){
				categories[i].per_page_discussion = false;
			}else{
				categories[i].per_page_discussion = null;
			}
		}
	}
	
};
