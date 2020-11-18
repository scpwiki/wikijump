

Wikijump.modules.AccountMessagesModule = {
	vars: {}
};

Wikijump.modules.AccountMessagesModule.listeners = {
	inbox: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("account/pm/PMInboxModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
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
		OZONE.ajax.requestModule("account/pm/PMSentModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
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
		OZONE.ajax.requestModule("account/pm/PMDraftsModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
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
		OZONE.ajax.requestModule("account/pm/PMComposeModule", p, Wikijump.modules.AccountMessagesModule.callbacks.compose);
		var tp = $("account-top-tabs");
		var as = tp.getElementsByTagName('a');
		for(var i=0; i<as.length; i++){
			YAHOO.util.Dom.removeClass(as[i], "active");
		}
		var curr = as.item(3);
		YAHOO.util.Dom.addClass(curr, "active");

		YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", Wikijump.modules.AccountMessagesModule.listeners.inbox);
	},

	viewInboxMessage: function(messageId){
		var p = new Object();
		p['message_id'] = messageId;
		OZONE.ajax.requestModule("account/pm/PMInboxMessageModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
	},
	replyInboxMessage: function(e, messageId){
		var p = new Object();
		if(messageId){
			p.replyMessageId = messageId;
		}
		OZONE.ajax.requestModule("account/pm/PMComposeModule", p, Wikijump.modules.AccountMessagesModule.callbacks.replyInboxMessage);

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
		OZONE.ajax.requestModule("account/pm/PMSentMessageModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
	},
	viewDraftsMessage: function(messageId){
		var p = new Object();
		p['message_id'] = messageId;
		OZONE.ajax.requestModule("account/pm/PMDraftsMessageModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
	},
	cancelCompose: function(e){

	}

}

Wikijump.modules.AccountMessagesModule.callbacks = {
	setActionArea: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("pm-action-area").innerHTML = r.body;
		// format dates
		OZONE.utils.formatDates($("pm-action-area"));
		Wikijump.page.fixers.fixEmails($("pm-action-area"));

	},
	compose: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		$("pm-action-area").innerHTML = r.body;
		// format dates
		OZONE.utils.formatDates($("pm-action-area"));
		if(r.toUserId){
			Wikijump.modules.AccountMessagesModule.vars.toUserId = r.toUserId;
			Wikijump.modules.AccountMessagesModule.vars.toUserName = r.toUserName;
		}else{
			Wikijump.modules.AccountMessagesModule.vars.toUserId = null;
			Wikijump.modules.AccountMessagesModule.vars.toUserName = null;
		}

	},
	replyInboxMessage: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		if(r.toUserId){
			Wikijump.modules.AccountMessagesModule.vars.toUserId = r.toUserId;
			Wikijump.modules.AccountMessagesModule.vars.toUserName = r.toUserName;
		}else{
			Wikijump.modules.AccountMessagesModule.vars.toUserId = null;
			Wikijump.modules.AccountMessagesModule.vars.toUserName = null;
		}
		$("pm-reply-area").innerHTML = r.body;
		setTimeout('OZONE.visuals.scrollTo($("pm-reply-area"))', 200);
		$("inbox-message-options").style.display = "none";
		var nav1 = $("inbox-message-nav");
		if(nav1){
			nav1.style.display = "none";
		}

		YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", Wikijump.modules.AccountMessagesModule.listeners.cancelReplyInboxMessage);
	}

}

Wikijump.modules.AccountMessagesModule.utils = {
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
	Wikijump.modules.AccountMessagesModule.listeners.inbox(null, pageNo);
}

function sentPage(pageNo){
	Wikijump.modules.AccountMessagesModule.listeners.sent(null, pageNo);
}
function draftsPage(pageNo){
	Wikijump.modules.AccountMessagesModule.listeners.drafts(null, pageNo);
}

Wikijump.modules.AccountMessagesModule.init = function(){
	if(window.composeTo){
		Wikijump.modules.AccountMessagesModule.listeners.compose(null,window.composeTo);
		// attach cancel listener ???
	}else if(window.inboxMessage){
		Wikijump.modules.AccountMessagesModule.listeners.viewInboxMessage(window.inboxMessage);
	}else{
		inboxPage();
	}
}

Wikijump.modules.AccountMessagesModule.init();
