

Wikijump.modules.ForumDeletePostModule = {};

Wikijump.modules.ForumDeletePostModule.listeners = {
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
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ForumDeletePostModule.callbacks.deletePost);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Deleting post...";
		w.show();
	}

}

Wikijump.modules.ForumDeletePostModule.callbacks = {
	deletePost: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The post has been deleted.";
		w.show();
		setTimeout('window.location.reload()', 1000);
	}

}
