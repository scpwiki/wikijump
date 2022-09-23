

Wikijump.modules.PMSentModule = {};

Wikijump.modules.PMSentModule.vars = {
	currentMessageId: null
}

Wikijump.modules.PMSentModule.listeners = {
	loadList: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("Account/PM/PMSentModule", p, Wikijump.modules.AccountMessagesModule.callbacks.setActionArea);
		if(e){	Wikijump.modules.AccountMessagesModule.utils.highlightTab(e);}
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
		p.event = 'removeSelectedSent';
		p.selected = JSON.stringify(selected);
		OZONE.ajax.requestModule(null, p, Wikijump.modules.PMSentModule.callbacks.removeSelected);

	},
	removeSentMessage: function(e, messageId){
		Wikijump.modules.PMSentModule.vars.currentMessageId = messageId;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = "Are sure you want to remove this message?";
		w.buttons = ['cancel', 'remove message'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('remove message', Wikijump.modules.PMSentModule.listeners.removeSentMessage2);
		w.focusButton = 'cancel';
		w.show();
	},

	removeSentMessage2: function(e, messageId){
		var p = new Object();
		p.action = "PMAction";
		p.event = 'removeSentMessage';
		p.message_id = Wikijump.modules.PMSentModule.vars.currentMessageId;
		OZONE.ajax.requestModule(null, p, Wikijump.modules.PMSentModule.callbacks.removeSentMessage);
	}
}

Wikijump.modules.PMSentModule.callbacks = {
	removeSelected: function(r){
		Wikijump.modules.PMSentModule.listeners.loadList(null, 1);
	},
	removeSentMessage: function(r){
		if(r.status == 'ok'){
			var w = new OZONE.dialogs.SuccessBox();
			w.content = "The message has been removed.";
			w.show();

			if(r.messageId){
				setTimeout('Wikijump.modules.AccountMessagesModule.listeners.viewSentMessage("'+r.messageId+'")', 1000);
			}else{
				// return to inbox view
				setTimeout('sentPage(1)');
			}
		}
	}
}

Wikijump.modules.PMSentModule.init = function(){

}
