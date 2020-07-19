

Wikijump.modules.AWForumModule = {};

Wikijump.modules.AWForumModule.listeners = {
	showWatchedThreads: function(e){
		OZONE.ajax.requestModule("/account/watch/AWThreadsListModule", null, Wikijump.modules.AWForumModule.callbacks.showWatchedThreads);
	},
	hideWatchedThreads: function(e){
		$("watched-threads-list").innerHTML="";
		$("watched-threads-list").style.display = "none";
		$("show-watched-threads-button").style.display="";
		$("hide-watched-threads-button").style.display="none";
	},

	removeWatchedThread: function(e, threadId){
		var p = new Object();
		p.threadId = threadId;
		p.action = "WatchAction";
		p.event = "removeWatchedThread";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AWForumModule.callbacks.removeWatchedThread);
	},
	updateList: function(event,pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}

		OZONE.ajax.requestModule("account/watch/AWForumListModule", p, Wikijump.modules.AWForumModule.callbacks.updateList);
	}
}

Wikijump.modules.AWForumModule.callbacks = {
	showWatchedThreads: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var el = $("watched-threads-list");
		el.innerHTML = r.body;
		el.style.display = "block";
		$("hide-watched-threads-button").style.display="";
		$("show-watched-threads-button").style.display="none";
	},

	removeWatchedThread: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Thread not being watched any more.";
		w.show();
		Wikijump.modules.AWForumModule.listeners.showWatchedThreads();
		Wikijump.modules.AWForumModule.listeners.updateList();
	},
	updateList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		$("watched-forum-list").innerHTML = r.body;
		OZONE.utils.formatDates("watched-forum-list");
	}

}
