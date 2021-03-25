

Wikijump.modules.ForumThreadMoveModule = {};

Wikijump.modules.ForumThreadMoveModule.listeners = {
	move: function(e){
		var categoryId = $("move-thread-category").value;
		var p = new Object();
		p.categoryId = categoryId;
		p.threadId = Wikijump.forumThreadId;
		p.action = 'ForumAction';
		p.event = 'moveThread';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ForumThreadMoveModule.callbacks.move);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Moving thread...";
		w.show();
	}
}

Wikijump.modules.ForumThreadMoveModule.callbacks = {
	move: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Thread has been moved.";
		w.show();
		setTimeout("window.location.reload()", 1000);
	}

}
