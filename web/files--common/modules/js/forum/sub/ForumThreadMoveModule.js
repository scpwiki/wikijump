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

WIKIDOT.modules.ForumThreadMoveModule = {};

WIKIDOT.modules.ForumThreadMoveModule.listeners = {
	move: function(e){
		var categoryId = $("move-thread-category").value;
		var p = new Object();
		p.categoryId = categoryId;
		p.threadId = WIKIDOT.forumThreadId;
		p.action = 'ForumAction';
		p.event = 'moveThread';
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ForumThreadMoveModule.callbacks.move);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Moving thread...";
		w.show();
	}	
}

WIKIDOT.modules.ForumThreadMoveModule.callbacks = {
	move: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Thread has been moved.";
		w.show();
		setTimeout("window.location.reload()", 1000);
	}
		
}