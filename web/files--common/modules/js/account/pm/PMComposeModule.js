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

WIKIDOT.modules.PMComposeModule = {};

WIKIDOT.modules.PMComposeModule.vars = {
	recipientId: null
}

WIKIDOT.modules.PMComposeModule.listeners = {
	changeRecipient: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		WIKIDOT.modules.PMComposeModule.vars.recipientId = null;
	},
	
	preview: function(e){
		var p = new Object();
		p['source'] = $("editor-textarea").value;
		p['subject'] = $("pm-subject").value;
		if(WIKIDOT.modules.PMComposeModule.vars.recipientId){
			p['to_user_id'] = WIKIDOT.modules.PMComposeModule.vars.recipientId;
		}
		OZONE.ajax.requestModule("account/pm/PMPreviewModule", p, WIKIDOT.modules.PMComposeModule.callbacks.preview);
	},
	
	saveDraft: function(e){
		var p = new Object();
		p['source'] = $("editor-textarea").value;
		p['subject'] = $("pm-subject").value;
		if(WIKIDOT.modules.PMComposeModule.vars.recipientId != null){
			p['to_user_id'] = WIKIDOT.modules.PMComposeModule.vars.recipientId;
		}
		p['action'] = "PMAction";
		p['event'] = "saveDraft";
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.PMComposeModule.callbacks.saveDraft);
	},
	
	send: function(e){
		
		if(WIKIDOT.modules.PMComposeModule.vars.recipientId == null){
			// no recipient!
			var d = new OZONE.dialogs.ErrorDialog()
			d.content = "The recipient of the message should be chosen ;-)";
			d.show();
			return;
		}
		
		var p = new Object();
		p['source'] = $("editor-textarea").value;
		p['subject'] = $("pm-subject").value;
		p['to_user_id'] = WIKIDOT.modules.PMComposeModule.vars.recipientId;
		p['action'] = "PMAction";
		p['event'] = "send";
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.PMComposeModule.callbacks.send);
	},
	cancel: function(e){
		// warning: need to check the context... is it a "reply" or standalone compose?
		WIKIDOT.Editor.shutDown();
	},
	
	showContactsList: function(e){
		OZONE.ajax.requestModule("account/pm/PMComposeContactsListModule", null, WIKIDOT.modules.PMComposeModule.callbacks.showContactsList);
	}
	
}

WIKIDOT.modules.PMComposeModule.callbacks = {
	preview: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("pm-preview-area", r.body);
		OZONE.visuals.scrollTo("pm-preview-area");
	},
	send: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Message has been sent.";
		w.show();
		setTimeout('inboxPage()', 1500);
	},
	saveDraft: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Draft has been saved.";
		w.show();
	},
	checkCan: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
	},
	showContactsList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.Dialog();
		w.content = r.body;
		w.show();
		
		// check for sizes
		var list = $("pm-contacts-list");
		if(list.offsetHeight>500){
			list.style.height="500px";
			list.style.overflow="auto";

			OZONE.dialog.factory.boxcontainer().centerContent();
		}
	}
}

WIKIDOT.modules.PMComposeModule.init = function(){
	// init autocomplete
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
		WIKIDOT.modules.PMComposeModule.utils.selectRecipient(userId, userName);
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
	
	if(WIKIDOT.modules.AccountMessagesModule.vars.toUserId){
		WIKIDOT.modules.PMComposeModule.utils.selectRecipient(WIKIDOT.modules.AccountMessagesModule.vars.toUserId,WIKIDOT.modules.AccountMessagesModule.vars.toUserName);
		WIKIDOT.modules.AccountMessagesModule.vars.toUserId = null;
		WIKIDOT.modules.AccountMessagesModule.vars.toUserName = null;
	}
	// init editor
	WIKIDOT.Editor.init("editor-textarea", "editor-panel");
	
	YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", WIKIDOT.modules.PMComposeModule.listeners.cancel);
}

WIKIDOT.modules.PMComposeModule.utils={
	selectRecipient: function(userId, userName){
		var userString = WIKIDOT.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		WIKIDOT.modules.PMComposeModule.vars.recipientId = userId;
		
		// also check for permission
		var p = new Object();
		p.userId = userId;
		p.action = "PMAction";
		p.event = "checkCan";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PMComposeModule.callbacks.checkCan);
	}	
}

WIKIDOT.modules.PMComposeModule.init();
