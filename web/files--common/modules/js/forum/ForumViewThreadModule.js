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

WIKIDOT.modules.ForumViewThreadModule = {};

WIKIDOT.modules.ForumViewThreadModule.vars = {
	
}

WIKIDOT.modules.ForumViewThreadModule.listeners = {
	togglePostFold: function(e, postId){
		fDiv = $("post-"+postId); // leave it global... nasty.
		
		var ofx = new fx.Opacity(fDiv,{duration: 100, onComplete: function(){
			if(fDiv.className.indexOf(' folded')>=0){
				fDiv.className = fDiv.className.replace(/ folded/,'');
			} else {
				fDiv.className += ' folded';	
			}
			
			var ofx = new fx.Opacity(fDiv,{duration: 100});
			ofx.setOpacity(0);
			ofx.custom(0,1)
		}});
		ofx.custom(1,0);
	},
	
	togglePostOptions: function(e, postId){
		var oDiv = $("post-options-"+postId);
		if(oDiv.style.display != "block"){
			var inner = $("post-options-template").innerHTML;
			inner = inner.replace(/%POST_ID%/g, postId);
			oDiv.innerHTML = inner;
			
			// modify permalink...
			var els = oDiv.getElementsByTagName('a');
			for(var i=0; i<els.length; i++){
				if(els[i].innerHTML == 'permalink'){
					els[i].href=$("post-options-permalink-template").innerHTML+postId;
				}
				
			}
			var ofx = new fx.Opacity(oDiv.id,{duration:200});
			ofx.setOpacity(0);
			oDiv.style.display = "block";
			ofx.custom(0,1);
		} else {
			var ofx = new fx.Opacity(oDiv.id,{duration:200});
			ofx.custom(1,0);
			setTimeout('document.getElementById("post-options-'+postId+'").style.display="none"', 300);
		}	
	},
	
	toggleThreadOptions: function(e){
		var el= $("thread-options-2")
		var ofx = new fx.Opacity(el,{duration:200});
		var t = YAHOO.util.Event.getTarget(e);
		if(el.style.display == 'none'){
			ofx.setOpacity(0);
			el.style.display = "block";
			ofx.custom(0,1);
			t.innerHTML = "- less options";
		}else{
			ofx.custom(1,0);
			t.innerHTML = "+ more options";
			setTimeout('$("thread-options-2").style.display="none"',200);
		}
	},
	
	showPermalink: function(e, postId){
		var w = new OZONE.dialogs.InfoDialog();
		w.style={width: "60em"};
		//w.content='<h1>Permanent link</h1><p>Permanent link for this post is:</p>' +
		
		var uri = window.location.href.replace(/#.*$/, '').replace(/\/$/,'');
		if(!WIKIREQUEST.info.requestPageName.match(/^forum:thread$/)){
			if(!uri.match(/comments\/show/)){
				uri += '/comments/show';
			}
		}
		uri += "#post-"+postId;
		
		w.content='<h1>Permanent link</h1><p>Permanent link for this post is:</p>' +
				'<p><strong>'+uri+'</strong></p>'	;
		
		w.show();
	},
	
	newPost: function(e, postId){
		if(WIKIDOT.Editor.editElementId){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content="You have an active editor somewhere already and it is not" +
					" possible to edit multiple elemnts at once.<br/><br/>" +
					'(<a href="javascript:;" onclick="OZONE.visuals.scrollTo(\''+WIKIDOT.Editor.editElementId+'\');OZONE.dialog.cleanAll()">scroll to active editor</a>)';
			w.show();
			return;
		}
		//alert(postId)
		// postId is an optional postId to reply to.
		var p = new Object();
		p.postId = postId;
		p.threadId = WIKIDOT.forumThreadId;
		OZONE.ajax.requestModule('forum/sub/ForumNewPostFormModule', p, WIKIDOT.modules.ForumViewThreadModule.callbacks.newPost);
	},
	editPost: function(e, postId){
		if(WIKIDOT.Editor.editElementId){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content="You have an active editor somewhere already and it is not" +
					" possible to edit multiple elemnts at once.<br/><br/>" +
					'(<a href="javascript:;" onclick="OZONE.visuals.scrollTo(\''+WIKIDOT.Editor.editElementId+'\');OZONE.dialog.cleanAll()">scroll to active editor</a>)';
			w.show();
			return;
		}
		var p = new Object();
		p.postId = postId;
		p.threadId = WIKIDOT.forumThreadId;
		OZONE.ajax.requestModule('forum/sub/ForumEditPostFormModule', p, WIKIDOT.modules.ForumViewThreadModule.callbacks.editPost);
		
	},
	
	deletePost: function(e, postId){
		
		OZONE.ajax.requestModule("forum/sub/ForumDeletePostModule", {postId: postId}, WIKIDOT.modules.ForumViewThreadModule.callbacks.deletePost);
		
	},
	
	foldAll: function(e){
		var posts = YAHOO.util.Dom.getElementsByClassName("post", 'div');
		for(var i=0; i<posts.length; i++){
			YAHOO.util.Dom.addClass(posts[i], "folded");
		}	
	},
	unfoldAll: function(e){
		var posts = YAHOO.util.Dom.getElementsByClassName("post", 'div');
		for(var i=0; i<posts.length; i++){
			YAHOO.util.Dom.removeClass(posts[i], "folded");
		}	
	},
	showHistory: function(e, postId){
		var p = new Object();
		p.postId = postId;
		OZONE.ajax.requestModule("forum/sub/ForumPostRevisionsModule", p, WIKIDOT.modules.ForumViewThreadModule.callbacks.showHistory);
	},
	hideHistory: function(e, postId){
		var postDiv = $("post-"+postId);
		var revDiv = YAHOO.util.Dom.getElementsByClassName('revisions', 'div', postDiv)[0];
		var chDiv = YAHOO.util.Dom.getElementsByClassName('changes', 'div', postDiv)[0];
		chDiv.style.display = "block"
		revDiv.style.display = "none";
	},
	showRevision: function(e, revisionId){
		var p = new Object();
		p.revisionId = revisionId;
		OZONE.ajax.requestModule("forum/sub/ForumPostRevisionModule", p, WIKIDOT.modules.ForumViewThreadModule.callbacks.showRevision);
		// clear active
		
		var t = YAHOO.util.Event.getTarget(e);
		
		var t2 = t.parentNode;
		while(!t2.tagName || t2.tagName.toLowerCase() != 'table'){
			t2 = t2.parentNode;
		}
		var tact =  YAHOO.util.Dom.getElementsByClassName('active', 'tr', t2)[0];
		YAHOO.util.Dom.removeClass(tact, 'active');
		
		while(!t.tagName || t.tagName.toLowerCase() != 'tr'){
			t = t.parentNode;
		}
		YAHOO.util.Dom.addClass(t, "active");
		
	},
	editThreadMeta: function(e){
		var p = new Object();
		p.threadId = WIKIDOT.forumThreadId;
		OZONE.ajax.requestModule("forum/sub/ForumEditThreadMetaModule", p, WIKIDOT.modules.ForumViewThreadModule.callbacks.editThreadMeta);
	},
	editThreadStickiness: function(e){
		var p = new Object();
		p.threadId = WIKIDOT.forumThreadId;
		OZONE.ajax.requestModule("forum/sub/ForumEditThreadStickinessModule", p, WIKIDOT.modules.ForumViewThreadModule.callbacks.editThreadStickiness);
	},
	editThreadBlock: function(e){
		var p = new Object();
		p.threadId = WIKIDOT.forumThreadId;
		OZONE.ajax.requestModule("forum/sub/ForumEditThreadBlockModule", p, WIKIDOT.modules.ForumViewThreadModule.callbacks.editThreadBlock);
	},
	moveThread: function(e){
		var p = new Object();
		p.threadId = WIKIDOT.forumThreadId;
		OZONE.ajax.requestModule("forum/sub/ForumThreadMoveModule", p, WIKIDOT.modules.ForumViewThreadModule.callbacks.moveThread);
	},
	
	watchThread: function(e){
		var p = new Object();
		p.threadId = WIKIDOT.forumThreadId;
		p.action = "WatchAction";
		p.event = "watchThread";
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ForumViewThreadModule.callbacks.watchThread);
	}
		
}

WIKIDOT.modules.ForumViewThreadModule.callbacks = {
	newPost: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		// proceed
		var parentId = r.parentId;
		
		var formDiv = document.createElement('div')
		formDiv.id="new-post-form-container";
		formDiv.innerHTML = r.body;
		// find the location for the form-div and insert....
		if(parentId == null){
			// append at the end of "forum-posts-container"
			var forumPostsContainer = $("thread-container");
			forumPostsContainer.appendChild(formDiv);
			// hide "new post" button
			$("new-post-button").style.display="none";
		} else {
			var postContainer = $("fpc-"+parentId);
			var post = $("post-"+parentId);
			if(r.parentChanged == true){
				postContainer.appendChild(formDiv);
				
			}else{
				OZONE.dom.insertAfter(postContainer,formDiv,post);	
			}
		}

		// init editor
		WIKIDOT.Editor.init("np-text", "np-editor-panel");
		
		setTimeout('OZONE.visuals.scrollTo("new-post-form-container")', 300);	
	},
	editPost: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var formDiv = document.createElement('div')
		formDiv.id="edit-post-form-container";
		formDiv.innerHTML = r.body;
		// now where to put it....
		var postContainer = $("fpc-"+r.postId);
		var post = $("post-"+r.postId); 
		OZONE.dom.insertAfter(postContainer,formDiv,post);	
		
		WIKIDOT.Editor.init("np-text", "np-editor-panel");
		setTimeout('OZONE.visuals.scrollTo("edit-post-form-container")', 300);	
	},
	
	showHistory: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var postDiv = $("post-"+r.postId);
		var revDiv = YAHOO.util.Dom.getElementsByClassName('revisions', 'div', postDiv)[0];
		var chDiv = YAHOO.util.Dom.getElementsByClassName('changes', 'div', postDiv)[0];
		chDiv.style.display = "none";
		
		revDiv.innerHTML = r.body;
		revDiv.style.display = "block";
		OZONE.utils.formatDates(revDiv);
		
	},
	showRevision: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var postId = r.postId;
		$("post-content-"+postId).innerHTML = r.content;
		$("post-title-"+postId).innerHTML = r.title;
		
	},
	
	deletePost: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}		
		var po = $("post-"+r.postId);
		var co = $("fpc-"+r.postId);
		YAHOO.util.Dom.addClass(co, "fordelete");
		var id = "delete-post-"+r.postId;
		if($(id)){
			$(id).parentNode.removeChild($(id));
		}
		po.innerHTML += r.body;
		OZONE.visuals.scrollTo(id);
	},
	
	editThreadMeta: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var el = $("thread-action-area");
		el.style.display = "block";
		el.innerHTML = r.body;
		var limiter = new OZONE.forms.lengthLimiter("thread-description", "desc-charleft", 1000);
	},
	editThreadStickiness: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var el = $("thread-action-area");
		el.style.display = "block";
		el.innerHTML = r.body;
	},
	editThreadBlock: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var el = $("thread-action-area");
		el.style.display = "block";
		el.innerHTML = r.body;
	},
	moveThread: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var el = $("thread-action-area");
		el.style.display = "block";
		el.innerHTML = r.body;
	},
	watchThread: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Thead added to watched.";
		w.show();
	}
	
}

// shortcut functions versions

togglePostOptions = function(e, postId){
	WIKIDOT.modules.ForumViewThreadModule.listeners.togglePostOptions(e, postId);	
}
togglePostFold = function(e, postId){
	WIKIDOT.modules.ForumViewThreadModule.listeners.togglePostFold(e, postId);
}	
postReply = function(e, postId){
	WIKIDOT.modules.ForumViewThreadModule.listeners.newPost(e, postId);
}
