

Wikijump.modules.ForumCommentsModule = {};

Wikijump.modules.ForumCommentsModule.listeners = {
	showComments: function(e){
		// if "thread-container" is filled with data - just show it.
		// if not - make an ajax request for the content
		var tc = $("thread-container");
		if(tc.innerHTML.match(/^[\s\n\r]*$/)){
			tc.innerHTML = '<div class="wait-block">Loading comments...</div>';
			var p = new Object();
			p.pageId = WIKIREQUEST.info.pageId;
			OZONE.ajax.requestModule("forum/ForumCommentsListModule", p, Wikijump.modules.ForumCommentsModule.callbacks.showComments);
		}else{
			tc.style.display="block";
			$("comments-options-hidden").style.display="none";
			$("comments-options-shown").style.display="block";
		}
	},

	hideComments: function(e){
		var tc = $("thread-container");
		tc.style.display="none";
		$("comments-options-hidden").style.display="block";
		$("comments-options-shown").style.display="none";
	}
}

Wikijump.modules.ForumCommentsModule.callbacks = {
	showComments: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		var tc = $("thread-container");
		OZONE.utils.setInnerHTMLContent(tc, r.body);
		tc.style.display="block";
		$("comments-options-hidden").style.display="none";
		$("comments-options-shown").style.display="block";

		Wikijump.forumThreadId = r.threadId;
	}
}
