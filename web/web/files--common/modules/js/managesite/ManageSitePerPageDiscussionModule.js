

Wikijump.modules.ManageSitePerPageDiscussionModule = {};

Wikijump.modules.ManageSitePerPageDiscussionModule.listeners ={
	save: function(e){
		Wikijump.modules.ManageSitePerPageDiscussionModule.utils.updateFromForm();
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		var parms = new Object();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteForumAction";
		parms['event'] = "savePerPageDiscussion";
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManageSitePerPageDiscussionModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
};

Wikijump.modules.ManageSitePerPageDiscussionModule.callbacks = {
	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
};

Wikijump.modules.ManageSitePerPageDiscussionModule.utils = {
	updateFromForm: function(){
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
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
