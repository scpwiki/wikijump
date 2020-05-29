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

WIKIDOT.modules.PMDraftsModule = {};

WIKIDOT.modules.PMDraftsModule.vars = {
	currentMessageId: null
}

WIKIDOT.modules.PMDraftsModule.listeners = {
	loadList: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("account/pm/PMDraftsModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
		if(e){	WIKIDOT.modules.AccountMessagesModule.utils.highlightTab(e);}
	},
	
	selectAll: function(e){
		var chs = YAHOO.util.Dom.getElementsByClassName("message-select");
		for(var i=0; i<chs.length; i++){
			chs[i].checked=true;
		}
		
	},
	
	removeSelected: function(e){
		var selected = new Array();
		var chs = YAHOO.util.Dom.getElementsByClassName("message-select");
		for(var i=0; i<chs.length; i++){
			if(chs[i].checked){
				selected.push(chs[i].id.replace(/message\-check\-/, ''));
			}
		}
		if(selected.length == 0){
			return;
		}
		var p = new Object();
		p.action = "PMAction";
		p.event = 'removeSelectedDrafts';
		p.selected = JSON.stringify(selected);
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PMDraftsModule.callbacks.removeSelected);
			
	},
	removeDraftsMessage: function(e, messageId){
		WIKIDOT.modules.PMDraftsModule.vars.currentMessageId = messageId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = "Are sure you want to remove this message?";
		w.buttons = ['cancel', 'remove message'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('remove message', WIKIDOT.modules.PMDraftsModule.listeners.removeDraftsMessage2);
		w.focusButton = 'cancel';
		w.show();
	},
	
	removeDraftsMessage2: function(e, messageId){
		var p = new Object();
		p.action = "PMAction";
		p.event = 'removeDraftsMessage';
		p.message_id = WIKIDOT.modules.PMDraftsModule.vars.currentMessageId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PMDraftsModule.callbacks.removeDraftsMessage);
	},
	editDraftMessage: function(e, messageId){
		var p = new Object();
		if(messageId){
			p.continueMessageId = messageId;
		}
		OZONE.ajax.requestModule("account/pm/PMComposeModule", p, WIKIDOT.modules.PMDraftsModule.callbacks.editDraftMessage);
		
	}
}

WIKIDOT.modules.PMDraftsModule.callbacks = {
	removeSelected: function(r){
		WIKIDOT.modules.PMDraftsModule.listeners.loadList(null, 1);
	},
	removeDraftsMessage: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessBox();
			w.content = "The message has been removed.";
			w.show();
			
			if(r.messageId){
				setTimeout('WIKIDOT.modules.AccountMessagesModule.listeners.viewDraftsMessage("'+r.messageId+'")', 1000);
			}else{
				// return to inbox view
				setTimeout('draftsPage(1)');
			}
		}
	},
	editDraftMessage: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		if(r.toUserId){
			WIKIDOT.modules.AccountMessagesModule.vars.toUserId = r.toUserId;
			WIKIDOT.modules.AccountMessagesModule.vars.toUserName = r.toUserName;
		}else{
			WIKIDOT.modules.AccountMessagesModule.vars.toUserId = null;
			WIKIDOT.modules.AccountMessagesModule.vars.toUserName = null;
		}
		$("pm-action-area").innerHTML = r.body;
		// format dates
		OZONE.utils.formatDates($("pm-action-area"));

		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = as.item(3);
		YAHOO.util.Dom.addClass(curr, "active");
		
		YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", WIKIDOT.modules.AccountMessagesModule.listeners.drafts);
	}
}

WIKIDOT.modules.PMDraftsModule.init = function(){
	
}
