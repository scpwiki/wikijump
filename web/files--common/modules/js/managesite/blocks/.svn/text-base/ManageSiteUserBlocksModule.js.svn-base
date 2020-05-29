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

WIKIDOT.modules.ManageSiteUserBlocksModule = {};

WIKIDOT.modules.ManageSiteUserBlocksModule.vars = {
	addFormInited: false,
	currentUserId: null,
	dCurrentUserId: null
}

WIKIDOT.modules.ManageSiteUserBlocksModule.listeners = {
	showAddForm: function(e){
		if(!WIKIDOT.modules.ManageSiteUserBlocksModule.vars.addFormInited){
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
				WIKIDOT.modules.ManageSiteUserBlocksModule.listeners.selectUser(userId, userName);
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
			var limiter = new OZONE.forms.lengthLimiter("user-block-reason", "reason-char-left", 200);

			WIKIDOT.modules.ManageSiteUserBlocksModule.vars.addFormInited = true;
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
		WIKIDOT.modules.ManageSiteUserBlocksModule.listeners.changeUser(null);
		
	},
	selectUser: function(userId, userName){
		var userString = WIKIDOT.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		WIKIDOT.modules.ManageSiteUserBlocksModule.vars.currentUserId = userId;
	},
	changeUser: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		WIKIDOT.modules.ManageSiteUserBlocksModule.vars.currentUserId = null;
	},
	
	blockUser: function(e){
		if(WIKIDOT.modules.ManageSiteUserBlocksModule.vars.currentUserId == null){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You must select a valid user to block.";
			w.show();
			return;
		}
		var p = new Object();
		p.userId = WIKIDOT.modules.ManageSiteUserBlocksModule.vars.currentUserId;
		p.reason = $("user-block-reason").value;
		p.action = "ManageSiteBlockAction";
		p.event = "blockUser";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteUserBlocksModule.callbacks.blockUser);
	},
	
	deleteBlock: function(e, userId, userName){
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.buttons = ['cancel', 'yes, delete block'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, delete block', WIKIDOT.modules.ManageSiteUserBlocksModule.listeners.deleteBlock2, userId);
		w.content = "Are you sure you want to remove the user block for the user <strong>"+userName+"</strong>?";
		w.show();
		WIKIDOT.modules.ManageSiteUserBlocksModule.vars.dCurrentUserId = userId;
	},
	deleteBlock2: function(e){
		var userId = WIKIDOT.modules.ManageSiteUserBlocksModule.vars.dCurrentUserId;
		var p = new Object();
		p.userId = userId;
		p.action = "ManageSiteBlockAction";
		p.event = "deleteBlock";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteUserBlocksModule.callbacks.deleteBlock);
	}
}

WIKIDOT.modules.ManageSiteUserBlocksModule.callbacks = {
	blockUser: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User blocked.";
		w.show();
		// refresh the screen too
		setTimeout('WIKIDOT.modules.ManagerSiteModule.utils.loadModule("sm-user-blocks")', 1500);
		
	},
	deleteBlock: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User block removed.";
		w.show();
		// refresh the screen too
		setTimeout('WIKIDOT.modules.ManagerSiteModule.utils.loadModule("sm-user-blocks")', 1500);
		
	}
}

WIKIDOT.modules.ManageSiteUserBlocksModule.init = function(){
	
}

WIKIDOT.modules.ManageSiteUserBlocksModule.init();
