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

WIKIDOT.modules.ASBlockedModule = {};

WIKIDOT.modules.ASBlockedModule.vars = {
	addFormInited: false,
	currentUserId: null,
	dCurrentUserId: null
}

WIKIDOT.modules.ASBlockedModule.listeners = {
	showAddForm: function(e){
		if(!WIKIDOT.modules.ASBlockedModule.vars.addFormInited){
			// init autocomplete now
			var dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users','name', 'user_id']); 
			dataSource.scriptQueryParam="q";
			dataSource.scriptQueryAppend = "&module=UserLookupQModule";
		
			var autoComp = new YAHOO.widget.AutoComplete('user-lookup','user-lookup-list', dataSource);

			autoComp.minQueryLength = 2;
			autoComp.queryDelay = 0.5;
			autoComp.forceSelection = true;
			autoComp.itemSelectEvent.subscribe(function(sType, args){
				var userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/,"$1");
				var userName = args[1].getElementsByTagName('div').item(0).innerHTML;
				WIKIDOT.modules.ASBlockedModule.listeners.selectUser(userId, userName);
			});
			
			autoComp.formatResult = function(aResultItem, sQuery) { 
				var name = aResultItem[0];
				var userId = aResultItem[1];
				if(name!= null){
					return '<div id="user-autocomplete-'+userId+'">'+name+'</div>';
				} else {
					return "";
				}
				
			}	
			
			WIKIDOT.modules.ASBlockedModule.vars.addFormInited = true;
		}
		$("show-add-block-button").style.display = "none";
		$("add-block-user-div").style.display = "block";
		OZONE.visuals.scrollTo("add-block-user-div");
	},
	
	cancelAdd: function(e){
		// resets the forms?
		$("show-add-block-button").style.display = "block";
		$("add-block-user-div").style.display = "none";
		$("user-lookup").value="";
		WIKIDOT.modules.ASBlockedModule.listeners.changeUser(null);
		
	},
	selectUser: function(userId, userName){
		var userString = WIKIDOT.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		WIKIDOT.modules.ASBlockedModule.vars.currentUserId = userId;
	},
	changeUser: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		WIKIDOT.modules.ASBlockedModule.vars.currentUserId = null;
	},
	
	blockUser: function(e){
		if(WIKIDOT.modules.ASBlockedModule.vars.currentUserId == null){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You must select a valid user to block.";
			w.show();
			return;
		}
		var p = new Object();
		p.userId = WIKIDOT.modules.ASBlockedModule.vars.currentUserId;
		p.action = "AccountSettingsAction";
		p.event = "blockUser";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASBlockedModule.callbacks.blockUser);
	},
	
	deleteBlock: function(e, userId, userName){
		var p = new Object();
		p.userId = userId;
		p.action = "AccountSettingsAction";
		p.event = "deleteBlock";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ASBlockedModule.callbacks.deleteBlock);
	}
}

WIKIDOT.modules.ASBlockedModule.callbacks = {
	blockUser: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User blocked.";
		w.show();
		// refresh the screen too
		setTimeout('OZONE.ajax.requestModule("account/settings/ASBlockedModule", null, WIKIDOT.modules.AccountModule.callbacks.menuClick)', 1500);
		
	},
	deleteBlock: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User block removed.";
		w.show();
		setTimeout('OZONE.ajax.requestModule("account/settings/ASBlockedModule", null, WIKIDOT.modules.AccountModule.callbacks.menuClick)', 1500);
	}
}

WIKIDOT.modules.ASBlockedModule.init = function(){
	
}

WIKIDOT.modules.ASBlockedModule.init();
