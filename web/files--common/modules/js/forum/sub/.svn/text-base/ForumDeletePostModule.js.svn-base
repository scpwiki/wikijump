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

WIKIDOT.modules.ForumDeletePostModule = {};

WIKIDOT.modules.ForumDeletePostModule.listeners = {
	cancel: function(e, postId){
		var co = $("fpc-"+postId);
		YAHOO.util.Dom.removeClass(co, "fordelete");
		var id = "delete-post-"+postId;
		if($(id)){
			$(id).parentNode.removeChild($(id));
		}
	},
	
	deletePost: function(e, postId){
		var p = new Object();
		p.action = "ForumAction";
		p.event = "deletePost";
		p.postId = postId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ForumDeletePostModule.callbacks.deletePost);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Deleting post...";
		w.show();
	}
	
}

WIKIDOT.modules.ForumDeletePostModule.callbacks = {
	deletePost: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The post has been deleted.";
		w.show();
		setTimeout('window.location.reload()', 1000);
	}
	
}