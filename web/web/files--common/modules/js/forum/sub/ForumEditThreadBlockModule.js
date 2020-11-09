

Wikijump.modules.ForumEditThreadBlockModule = {}

Wikijump.modules.ForumEditThreadBlockModule.listeners = {
	save: function(e){
		var p = new Object();
		p.threadId = Wikijump.forumThreadId;
		if($("thread-block-checkbox").checked){
			p.block = true;
		}
		p.action = 'ForumAction';
		p.event = 'saveBlock';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ForumEditThreadBlockModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
}

Wikijump.modules.ForumEditThreadBlockModule.callbacks = {
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
