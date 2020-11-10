

Wikijump.modules.ForumEditThreadStickinessModule = {}

Wikijump.modules.ForumEditThreadStickinessModule.listeners = {
	save: function(e){
		var p = new Object();
		p.threadId = Wikijump.forumThreadId;
		if($("thread-sticky-checkbox").checked){
			p.sticky = true;
		}
		p.action = 'ForumAction';
		p.event = 'saveSticky';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ForumEditThreadStickinessModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
}

Wikijump.modules.ForumEditThreadStickinessModule.callbacks = {
	save: function(r){
		if(r.status != 'ok'){
			var w = new OZONE.dialogs.ErrorDialog();	w.content = r.message; w.show();	return;
		}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your changes have been saved.";
		w.show();

		setTimeout('window.location.hash=""; window.location.reload()', 1000);

	}
}
