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

WIKIDOT.modules.AccountMessagesModule = {
	vars: {}
};

WIKIDOT.modules.AccountMessagesModule.listeners = {
	inbox: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("account/pm/PMInboxModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = as.item(0);
		
		YAHOO.util.Dom.addClass(curr, "active");
		
	},
	
	sent: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("account/pm/PMSentModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = as.item(1);
		YAHOO.util.Dom.addClass(curr, "active");
	},
	drafts: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("account/pm/PMDraftsModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = as.item(2);
		YAHOO.util.Dom.addClass(curr, "active");
	},
	
	compose: function(e, userId){
		var p = new Object();
		if(userId != null){
			p['toUserId'] = userId;
		}
		OZONE.ajax.requestModule("account/pm/PMComposeModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.compose);
		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = as.item(3);
		YAHOO.util.Dom.addClass(curr, "active");
		
		YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", WIKIDOT.modules.AccountMessagesModule.listeners.inbox);
	},
	
	viewInboxMessage: function(messageId){
		var p = new Object();
		p['message_id'] = messageId;
		OZONE.ajax.requestModule("account/pm/PMInboxMessageModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
	},
	replyInboxMessage: function(e, messageId){
		var p = new Object();
		if(messageId){
			p.replyMessageId = messageId;
		}
		OZONE.ajax.requestModule("account/pm/PMComposeModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.replyInboxMessage);
		
	},
	
	cancelReplyInboxMessage: function(e){
		$("pm-reply-area").innerHTML = "";
		$("inbox-message-options").style.display = "block";
		var nav1 = $("inbox-message-nav");
		if(nav1){
			nav1.style.display = "block";
		}
	},
	viewSentMessage: function(messageId){
		var p = new Object();
		p['message_id'] = messageId;
		OZONE.ajax.requestModule("account/pm/PMSentMessageModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
	},
	viewDraftsMessage: function(messageId){
		var p = new Object();
		p['message_id'] = messageId;
		OZONE.ajax.requestModule("account/pm/PMDraftsMessageModule", p, WIKIDOT.modules.AccountMessagesModule.callbacks.setActionArea);
	},
	cancelCompose: function(e){
		
	}

}

WIKIDOT.modules.AccountMessagesModule.callbacks = {
	setActionArea: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("pm-action-area").innerHTML = r.body;
		// format dates
		OZONE.utils.formatDates($("pm-action-area"));
		WIKIDOT.page.fixers.fixEmails($("pm-action-area"));

	},
	compose: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("pm-action-area").innerHTML = r.body;
		// format dates
		OZONE.utils.formatDates($("pm-action-area"));
		if(r.toUserId){
			WIKIDOT.modules.AccountMessagesModule.vars.toUserId = r.toUserId;
			WIKIDOT.modules.AccountMessagesModule.vars.toUserName = r.toUserName;
		}else{
			WIKIDOT.modules.AccountMessagesModule.vars.toUserId = null;
			WIKIDOT.modules.AccountMessagesModule.vars.toUserName = null;
		}
		
	},
	replyInboxMessage: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		if(r.toUserId){
			WIKIDOT.modules.AccountMessagesModule.vars.toUserId = r.toUserId;
			WIKIDOT.modules.AccountMessagesModule.vars.toUserName = r.toUserName;
		}else{
			WIKIDOT.modules.AccountMessagesModule.vars.toUserId = null;
			WIKIDOT.modules.AccountMessagesModule.vars.toUserName = null;
		}
		$("pm-reply-area").innerHTML = r.body;
		setTimeout('OZONE.visuals.scrollTo($("pm-reply-area"))', 200);
		$("inbox-message-options").style.display = "none";
		var nav1 = $("inbox-message-nav");
		if(nav1){
			nav1.style.display = "none";
		}
		
		YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", WIKIDOT.modules.AccountMessagesModule.listeners.cancelReplyInboxMessage);
	}

}

WIKIDOT.modules.AccountMessagesModule.utils = {
	highlightTab: function(e){
		// dehighlight all tabs
		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = YAHOO.util.Event.getTarget(e);
		if(curr.tagName.toLowerCase() == 'a'){
			YAHOO.util.Dom.addClass(curr, "active");
		}
	}
}

function inboxPage(pageNo){
	WIKIDOT.modules.AccountMessagesModule.listeners.inbox(null, pageNo);	
}

function sentPage(pageNo){
	WIKIDOT.modules.AccountMessagesModule.listeners.sent(null, pageNo);	
}
function draftsPage(pageNo){
	WIKIDOT.modules.AccountMessagesModule.listeners.drafts(null, pageNo);	
}

WIKIDOT.modules.AccountMessagesModule.init = function(){
	if(window.composeTo){
		WIKIDOT.modules.AccountMessagesModule.listeners.compose(null,window.composeTo);
		// attach cancel listener ???
	}else if(window.inboxMessage){
		WIKIDOT.modules.AccountMessagesModule.listeners.viewInboxMessage(window.inboxMessage);
	}else{
		inboxPage();
	}
}

WIKIDOT.modules.AccountMessagesModule.init();
