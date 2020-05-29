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

WIKIDOT.modules.ManageSiteForumPermissionsModule = {};

WIKIDOT.modules.ManageSiteForumPermissionsModule.vars = {
	users: ['a', // anonymous
			 'r', // registered at wikidot
			 'm', // member of the site
			 'o'	  // author of the post
			 ],
	permissions: ['t', // start new threads
				  'p', // add new posts
				  'e', // edit posts/threads (!!!)
				  's' // split - create new threads from existing posts
				   ]
}

WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners = {
	categoryChange: function(e){
		// update permissions info
		var categoryId = $("sm-perms-cats").value;
		var pstring;
		if(categoryId == ''){
			// default permissions
			$("sm-perms-noind").style.display = "none";
			$("sm-perms-table").style.display = "";
			pstring = WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.defaultPermissions;
		} else {
			$("sm-perms-noind").style.display = "block";
			var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
			if(category['permissions'] == null){
				$("sm-perms-noin").checked=true;
				$("sm-perms-table").style.display = "none";
				pstring = WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.defaultPermissions;
			} else {
				$("sm-perms-noin").checked=false;
				$("sm-perms-table").style.display = "block";
				pstring = category['permissions'];
			}
			
		}
		
		WIKIDOT.modules.ManageSiteForumPermissionsModule.utils.decodePermissions(pstring);
		return;
		WIKIDOT.modules.ManageSitePermissionsModule.utils.updateThemePreview();
	
	}	,
	
	indClick: function(e){
		var categoryId = $("sm-perms-cats").value;
		if(categoryId == "") return; // should not be
		
		var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		if($("sm-perms-noin").checked == true){
			$("sm-perms-table").style.display = "none";
			category['permissions'] = null;
		}else{
			$("sm-perms-table").style.display = "";
			category['permissions'] = WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.defaultPermissions;
		}
	},
	
	permissionChange: function(e){

		// fix permissions first (difficult?)
		var target = YAHOO.util.Event.getTarget(e);
		 WIKIDOT.modules.ManageSiteForumPermissionsModule.utils.fixPermissions(target.id);
		// encode permissions and save
		var pstring = WIKIDOT.modules.ManageSiteForumPermissionsModule.utils.encodePermissions();
		// save changes to the array
		var categoryId = $("sm-perms-cats").value;
		if(categoryId == ''){
			WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.defaultPermissions = pstring;
		} else {
			var category = WIKIDOT.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
			category['permissions'] = pstring;
		}
		
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
		parms['default_permissions'] = WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.defaultPermissions;
		parms['action'] = "ManageSiteForumAction";
		parms['event'] = "saveForumPermissions";
		OZONE.ajax.requestModule("Empty", parms, WIKIDOT.modules.ManageSiteForumPermissionsModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving permissions...";
		w.show();
	}
}

WIKIDOT.modules.ManageSiteForumPermissionsModule.callbacks = {
	cancel: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("site-manager", r.body);
	},
	
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Permissions have been saved.";
		w.show();
	}
}

WIKIDOT.modules.ManageSiteForumPermissionsModule.utils = {
	encodePermissions: function(){
		var farray = OZONE.utils.formToArray("sm-perms-form");
		
		// now traverse the form...
		var users = WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.users;
		var permissions =  WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.permissions;
		var i,j;
		var out=''; // output
		var tag;
		
		for(i=0; i<permissions.length; i++){
			if(i>0){out +=";";}
			out += permissions[i]+':';
			for(j=0;j<users.length; j++){
				// find a checkbox and check value
				tag = "sm-"+permissions[i]+'-'+users[j];
				if($(tag) && $(tag).checked == true){
					out += users[j];
				}
			}
		}
		return out;
	},
	
	decodePermissions: function(pstring){
		
		var activName;
		var activPerms;
		var activUser;
		var tag;
		var el;
		
		var form = document.getElementById("sm-perms-form");
		// clear the table
		var users = WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.users;
		var permissions =  WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.permissions;
		var i,j;
		var out='';
		for(i=0; i<permissions.length; i++){
			if(i>0){out +=";";}
			out += permissions[i]+':';
			for(j=0;j<users.length; j++){
				tag = 'sm-'+permissions[i]+'-'+users[j];
				el = $(tag);
				if(el){el.checked = false;}
			}
		}
		if(pstring != null && pstring != ''){
			
			var activs = pstring.split(';');
			for(i=0; i<activs.length; i++){
				var activs2 = activs[i].split(':');
				activName = activs2[0];
				activPerms = activs2[1];
				for(j=0;j<activPerms.length;j++){
					activUser = activPerms.charAt(j);
					// now set the checkbox
					tag = 'sm-'+activName+'-'+activUser;
					el = $(tag);
					if(el){el.checked = true;}
				}
			}
		}
	},
	
	fixPermissions: function(id){
		// an ugly way...
		
		var tag, el2,el;
		
		var el = $(id);
		
		var tsplit = id.split("-");
		var activ = tsplit[1];
		var user = tsplit[2];
		var charray = new Array();
		if(el.checked == true){
			switch(user){
				case "a":
					charray = ['r','m'];
					break;
				case 'r':
					charray = ['m'];
					break;
				case 'm':
					charray = [];
					break;
			}
			for(i=0;i<charray.length;i++){
				tag = 'sm-'+activ+'-'+charray[i];
				el2 = document.getElementById(tag);
				if(el2){el2.checked = true;}
			}
		}
		if(el.checked == false){
			switch(user){
				case "r":
					charray = ['a'];
					break;
				case 'm':
					charray = ['a', 'r'];
					break;
				case 'a':
					charray = [];
					break;
			}
			for(i=0;i<charray.length;i++){
				tag = 'sm-'+activ+'-'+charray[i];
				el2 = document.getElementById(tag);
				if(el2){el2.checked = false;}
			}
		
		}

	}
}

WIKIDOT.modules.ManageSiteForumPermissionsModule.init = function(){
	YAHOO.util.Event.addListener("sm-perms-cats", "change", WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners.categoryChange);
	WIKIDOT.modules.ManageSiteForumPermissionsModule.vars.defaultPermissions = $("default-forum-permissions").value;
	YAHOO.util.Event.addListener("sm-perms-noind", "click", WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners.indClick);
	YAHOO.util.Event.addListener("sm-perms-form", "click",  WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners.permissionChange);
	YAHOO.util.Event.addListener("sm-perms-cancel", "click", WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-perms-save", "click", WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners.save);
	
	WIKIDOT.modules.ManageSiteForumPermissionsModule.listeners.categoryChange(null);
}
WIKIDOT.modules.ManageSiteForumPermissionsModule.init();
