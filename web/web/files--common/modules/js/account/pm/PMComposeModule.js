

Wikijump.modules.PMComposeModule = {};

Wikijump.modules.PMComposeModule.vars = {
	recipientId: null
}

Wikijump.modules.PMComposeModule.listeners = {
	changeRecipient: function(e){
		$("select-user-div").style.display="block";
		$("selected-user-div").style.display="none";
		$("user-lookup").value="";
		Wikijump.modules.PMComposeModule.vars.recipientId = null;
	},

	preview: function(e){
		var p = new Object();
		p['source'] = $("editor-textarea").value;
		p['subject'] = $("pm-subject").value;
		if(Wikijump.modules.PMComposeModule.vars.recipientId){
			p['to_user_id'] = Wikijump.modules.PMComposeModule.vars.recipientId;
		}
		OZONE.ajax.requestModule("account/pm/PMPreviewModule", p, Wikijump.modules.PMComposeModule.callbacks.preview);
	},

	saveDraft: function(e){
		var p = new Object();
		p['source'] = $("editor-textarea").value;
		p['subject'] = $("pm-subject").value;
		if(Wikijump.modules.PMComposeModule.vars.recipientId != null){
			p['to_user_id'] = Wikijump.modules.PMComposeModule.vars.recipientId;
		}
		p['action'] = "PMAction";
		p['event'] = "saveDraft";
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.PMComposeModule.callbacks.saveDraft);
	},

	send: function(e){

		if(Wikijump.modules.PMComposeModule.vars.recipientId == null){
			// no recipient!
			var d = new OZONE.dialogs.ErrorDialog()
			d.content = "You must choose a recipient for this message.";
			d.show();
			return;
		}

		var p = new Object();
		p['source'] = $("editor-textarea").value;
		p['subject'] = $("pm-subject").value;
		p['to_user_id'] = Wikijump.modules.PMComposeModule.vars.recipientId;
		p['action'] = "PMAction";
		p['event'] = "send";
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.PMComposeModule.callbacks.send);
	},
	cancel: function(e){
		// warning: need to check the context... is it a "reply" or standalone compose?
		Wikijump.Editor.shutDown();
	},

	showContactsList: function(e){
		OZONE.ajax.requestModule("account/pm/PMComposeContactsListModule", null, Wikijump.modules.PMComposeModule.callbacks.showContactsList);
	}

}

Wikijump.modules.PMComposeModule.callbacks = {
	preview: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("pm-preview-area", r.body);
		OZONE.visuals.scrollTo("pm-preview-area");
	},
	send: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Message has been sent.";
		w.show();
		setTimeout('inboxPage()', 1500);
	},
	saveDraft: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Draft has been saved.";
		w.show();
	},
	checkCan: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
	},
	showContactsList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
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

Wikijump.modules.PMComposeModule.init = function(){
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
		Wikijump.modules.PMComposeModule.utils.selectRecipient(userId, userName);
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

	if(Wikijump.modules.AccountMessagesModule.vars.toUserId){
		Wikijump.modules.PMComposeModule.utils.selectRecipient(Wikijump.modules.AccountMessagesModule.vars.toUserId,Wikijump.modules.AccountMessagesModule.vars.toUserName);
		Wikijump.modules.AccountMessagesModule.vars.toUserId = null;
		Wikijump.modules.AccountMessagesModule.vars.toUserName = null;
	}
	// init editor
	Wikijump.Editor.init("editor-textarea", "editor-panel");

	YAHOO.util.Event.addListener("pm-compose-cancel-button", "click", Wikijump.modules.PMComposeModule.listeners.cancel);
}

Wikijump.modules.PMComposeModule.utils={
	selectRecipient: function(userId, userName){
		var userString = Wikijump.render.printuser(userId,userName, true);
		$("select-user-div").style.display="none";
		$("selected-user-div").style.display="block";
		$("selected-user-rendered").innerHTML = userString;
		Wikijump.modules.PMComposeModule.vars.recipientId = userId;

		// also check for permission
		var p = new Object();
		p.userId = userId;
		p.action = "PMAction";
		p.event = "checkCan";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.PMComposeModule.callbacks.checkCan);
	}
}

Wikijump.modules.PMComposeModule.init();
