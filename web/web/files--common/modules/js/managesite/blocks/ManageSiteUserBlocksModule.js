

Wikijump.modules.ManageSiteUserBlocksModule = {};

Wikijump.modules.ManageSiteUserBlocksModule.vars = {
	addFormInited: false,
	currentUserId: null,
	dCurrentUserId: null
}

Wikijump.modules.ManageSiteUserBlocksModule.listeners = {
	showAddForm: function(e){
		if(!Wikijump.modules.ManageSiteUserBlocksModule.vars.addFormInited){
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
				Wikijump.modules.ManageSiteUserBlocksModule.listeners.selectUser(userId, userName);
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

			Wikijump.modules.ManageSiteUserBlocksModule.vars.addFormInited = true;
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
		Wikijump.modules.ManageSiteUserBlocksModule.listeners.changeUser(null);

	},
	selectUser: function(userId, userName){
		var userString = Wikijump.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		Wikijump.modules.ManageSiteUserBlocksModule.vars.currentUserId = userId;
	},
	changeUser: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		Wikijump.modules.ManageSiteUserBlocksModule.vars.currentUserId = null;
	},

	blockUser: function(e){
		if(Wikijump.modules.ManageSiteUserBlocksModule.vars.currentUserId == null){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "You must select a valid user to block.";
			w.show();
			return;
		}
		var p = new Object();
		p.userId = Wikijump.modules.ManageSiteUserBlocksModule.vars.currentUserId;
		p.reason = $("user-block-reason").value;
		p.action = "ManageSiteBlockAction";
		p.event = "blockUser";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteUserBlocksModule.callbacks.blockUser);
	},

	deleteBlock: function(e, userId, userName){
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.buttons = ['cancel', 'yes, delete block'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, delete block', Wikijump.modules.ManageSiteUserBlocksModule.listeners.deleteBlock2, userId);
		w.content = "Are you sure you want to remove the user block for the user <strong>"+userName+"</strong>?";
		w.show();
		Wikijump.modules.ManageSiteUserBlocksModule.vars.dCurrentUserId = userId;
	},
	deleteBlock2: function(e){
		var userId = Wikijump.modules.ManageSiteUserBlocksModule.vars.dCurrentUserId;
		var p = new Object();
		p.userId = userId;
		p.action = "ManageSiteBlockAction";
		p.event = "deleteBlock";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteUserBlocksModule.callbacks.deleteBlock);
	}
}

Wikijump.modules.ManageSiteUserBlocksModule.callbacks = {
	blockUser: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User blocked.";
		w.show();
		// refresh the screen too
		setTimeout('Wikijump.modules.ManagerSiteModule.utils.loadModule("sm-user-blocks")', 1500);

	},
	deleteBlock: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "User block removed.";
		w.show();
		// refresh the screen too
		setTimeout('Wikijump.modules.ManagerSiteModule.utils.loadModule("sm-user-blocks")', 1500);

	}
}

Wikijump.modules.ManageSiteUserBlocksModule.init = function(){

}

Wikijump.modules.ManageSiteUserBlocksModule.init();
