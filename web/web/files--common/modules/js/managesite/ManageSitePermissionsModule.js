

Wikijump.modules.ManagerSitePermissionsModule = {};

Wikijump.modules.ManagerSitePermissionsModule.vars = {
	users: ['a', // anonymous
			 'r', // registered at Wikijump
			 'm', // member of the site
			 'o'	  // owner (creator) of the page
			 ],
	permissions: [// 'v', // view page
				   'e', // edit page
				   'c', // create new pages
				   'm', // move pages
				   'd', // delete pages
				   'a', // attach files
				   'r', // rename files
				   'z', // replace/move/delete files
				   'o'  // show page options to...
				   ]
}

Wikijump.modules.ManagerSitePermissionsModule.listeners = {
	categoryChange: function(e){
		// update permissions info
		var categoryId = document.getElementById("sm-perms-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);
		// check if has a individual permissions
		if(category['name'] == "_default"){
			$("sm-perms-noind").style.display = "none";
			$("sm-perms-table").style.display = "";
		} else {
			$("sm-perms-noind").style.display = "block";
			if(category['permissions_default'] == true){
				$("sm-perms-noin").checked=true;
				$("sm-perms-table").style.display = "none";
			} else {
				$("sm-perms-noin").checked=false;
				$("sm-perms-table").style.display = "";
			}
		}
		var pstring = category['permissions'];
		if((pstring == null || pstring =='') && category['name'] != "_default"){
			// get a string from default category
			var defcat = Wikijump.modules.ManagerSiteModule.utils.getCategoryByName("_default");
			pstring = defcat['permissions'];
		}
		Wikijump.modules.ManagerSitePermissionsModule.utils.decodePermissions(pstring);
		return;
		Wikijump.modules.ManagerSitePermissionsModule.utils.updateThemePreview();

	},

	indClick: function(e){
		var categoryId = $("sm-perms-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		if($("sm-perms-noin").checked == true){
			$("sm-perms-table").style.display = "none";
			category['permissions_default'] = true;
		}else{
			$("sm-perms-table").style.display = "";
			category['permissions_default'] = false;
		}
	},

	permissionChange: function(e){

		// save changes to the array
		var categoryId = $("sm-perms-cats").value;
		var category = Wikijump.modules.ManagerSiteModule.utils.getCategoryById(categoryId);

		// fix permissions first (difficult?)
		var target = YAHOO.util.Event.getTarget(e);
		Wikijump.modules.ManagerSitePermissionsModule.utils.fixPermissions(target.id);
		// encode permissions and save
		var pstring = Wikijump.modules.ManagerSitePermissionsModule.utils.encodePermissions();
		category['permissions'] = pstring;

	},

	cancel: function(e){
		Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
	},

	save: function(e){
		// ok, do it the easy way: serialize categories using the JSON method
		var categories = Wikijump.modules.ManagerSiteModule.vars.categories;
		var serialized = JSON.stringify(categories);
		parms = new Array();
		parms['categories'] = serialized;
		parms['action'] = "ManageSiteAction";
		parms['event'] = "savePermissions";
		OZONE.ajax.requestModule("Empty", parms, Wikijump.modules.ManagerSitePermissionsModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving permissions...";
		w.show();
	}

}

Wikijump.modules.ManagerSitePermissionsModule.callbacks = {
	cancel: function(response){
		OZONE.utils.setInnerHTMLContent("site-manager", response.body);
	},

	save: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Permissions have been saved.";
		w.show();
	}

}

Wikijump.modules.ManagerSitePermissionsModule.utils = {
	encodePermissions: function(){
		var farray = OZONE.utils.formToArray("sm-perms-form");
		// now traverse the form...
		var users = Wikijump.modules.ManagerSitePermissionsModule.vars.users;
		var permissions =  Wikijump.modules.ManagerSitePermissionsModule.vars.permissions;
		var i,j;
		var out=''; // output
		var tag;
		for(i=0; i<permissions.length; i++){
			if(i>0){out +=";";}
			out += permissions[i]+':';
			for(j=0;j<users.length; j++){
				tag = permissions[i]+'-'+users[j];
				// find a checkbox and check value
				if(farray[tag] == 'on'){
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
		var users = Wikijump.modules.ManagerSitePermissionsModule.vars.users;
		var permissions =  Wikijump.modules.ManagerSitePermissionsModule.vars.permissions;
		var i,j;
		var out='';
		for(i=0; i<permissions.length; i++){
			if(i>0){out +=";";}
			out += permissions[i]+':';
			for(j=0;j<users.length; j++){
				tag = 'sm-'+permissions[i]+'-'+users[j];
				el = document.getElementById(tag);
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
					el = document.getElementById(tag);
					if(el){el.checked = true;}
				}
			}
		}
	},

	fixPermissions: function(id){
		// an ugly way...
		var el = $(id);

		var tsplit = id.split("-");
		var activ = tsplit[1];
		var user = tsplit[2];
		var charray = [];
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

		/*
		var users = Wikijump.modules.ManagerSitePermissionsModule.vars.users;
		var permissions =  Wikijump.modules.ManagerSitePermissionsModule.vars.permissions;
		var tag;
		var can;
		for(i=0; i<permissions.length; i++){
			can = false;
			for(j=0;j<users.length; j++){
				tag = 'sm-'+permissions[i]+'-'+users[j];
				el = document.getElementById(tag);
				if(el){
					if(el.checked == false && can == true){
						el.checked = true;
					}else if(el.checked ==)

				}
			}
		}
		*/
	}
}

Wikijump.modules.ManagerSitePermissionsModule.init = function(){
	YAHOO.util.Event.addListener("sm-perms-cats", "change", Wikijump.modules.ManagerSitePermissionsModule.listeners.categoryChange);
	YAHOO.util.Event.addListener("sm-perms-noind", "click", Wikijump.modules.ManagerSitePermissionsModule.listeners.indClick);

	YAHOO.util.Event.addListener("sm-perms-form", "click",  Wikijump.modules.ManagerSitePermissionsModule.listeners.permissionChange);
	// do it the other way...

	YAHOO.util.Event.addListener("sm-perms-cancel", "click", Wikijump.modules.ManagerSitePermissionsModule.listeners.cancel);
	YAHOO.util.Event.addListener("sm-perms-save", "click", Wikijump.modules.ManagerSitePermissionsModule.listeners.save);
	// init categories info
	Wikijump.modules.ManagerSitePermissionsModule.listeners.categoryChange(null);
}

Wikijump.modules.ManagerSitePermissionsModule.init();
