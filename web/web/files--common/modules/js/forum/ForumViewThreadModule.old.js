

togglePostOptions = function(event,postId){
	Wikijump.modules.ForumViewThreadModule.listeners.togglePostOptions(event,postId);
}
togglePostFold = function(event,postId){
	Wikijump.modules.ForumViewThreadModule.listeners.togglePostFold(event,postId);
}
reply = function(postId, parentId){
	Wikijump.modules.ForumViewThreadModule.listeners.newPost(postId, parentId);
}

Wikijump.modules.ForumViewThreadModule = {}

Wikijump.modules.ForumViewThreadModule.vars = {
	currentParentId: null, //id of the current parrent post
	clickedReplyPostId: null, // which post has been clicked with reply
	fakedParent: false, // if clicked != parent
	editActive: false,
	currentEditPostId: null
}

Wikijump.modules.ForumViewThreadModule.listeners = {
	newPost: function(postId, parentId){
		if(Wikijump.Editor.editElementId){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content="You have an active editor somewhere already and it is not" +
					" possible to edit multiple elemnts at once.<br/><br/>" +
					'(<a href="javascript:;" onclick="OZONE.visuals.scrollTo(\''+Wikijump.Editor.editElementId+'\');OZONE.dialog.cleanAll()">scroll to active editor</a>)';
			w.show();
			return;
		}
		if(Wikijump.modules.ForumViewThreadModule.vars.editActive){
			// it means an active post edit is somewhere...

			var t2 = new OZONE.dialogs.Dialog();
			t2.content='<h1>New post form already open</h1>' +
					'<p>It seems that you already are writing a new post or reply. ' +
					' Before starting a new edit please close (cancel or save) ' +
					'your active one.</p>';

			t2.buttons = [ "go to active edit", "close"];
			t2.title=" ";
			t2.clickOutsideToClose = true;
			t2.addButtonListener("go to active edit", Wikijump.modules.ForumViewThreadModule.listeners.goToActiveEdit);
			t2.addButtonListener("close", t2.close);
			t2.show();
			return;
		}
		// get new post form template and insert where necessary..

		var formInner = $("new-post-t").innerHTML;
		formInner = formInner.replace(/id="/g, 'id="p-');

		var formDiv = document.createElement('div')
		formDiv.id="new-post-form-container";
		formDiv.innerHTML = formInner;
		// find the location for the form-div and insert....
		if(postId == null){
			// append at the end of "forum-posts-container"
			var forumPostsContainer = $("thread-container");
			forumPostsContainer.appendChild(formDiv);
			// hide "new post" button
			$("new-post-button").style.display="none";
		} else {
			var postContainer = $("fpc-"+postId);
			var post = $("post-"+postId);
			OZONE.dom.insertAfter(postContainer,formDiv,post);
		}

		// save current parent id:
		if(parentId){
			Wikijump.modules.ForumViewThreadModule.vars.fakedParent = true;
			Wikijump.modules.ForumViewThreadModule.vars.currentParentId = parentId;
		}else{
			Wikijump.modules.ForumViewThreadModule.vars.currentParentId = postId;
			Wikijump.modules.ForumViewThreadModule.vars.fakedParent = false;
		}
		Wikijump.modules.ForumViewThreadModule.vars.editActive = true;

		// init editor
		Wikijump.Editor.init("p-np-text", "p-np-editor-panel");

		setTimeout('OZONE.visuals.scrollTo("new-post-form-container")', 300);
	},

	reply: function(postId){
		// locate a place to put edit box...
		var postContainer = $("fpc-"+postId);
		var post = $("post-"+postId);
		var editNode = document.createElement('div');

		var formInner = $("new-post-t").innerHTML;
		formInner = formInner.replace(/id="/g, 'id="p-');

		editNode.innerHTML = "";
		OZONE.dom.insertAfter(postContainer,editNode,post);

		OZONE.visuals.scrollTo(editNode);
	},

	cancel: function(e){
		// remove form
		var formDiv = $('new-post-form-container');
		formDiv.parentNode.removeChild(formDiv);
		document.getElementById("new-post-button").style.display="";
		Wikijump.modules.ForumViewThreadModule.vars.currentParentId = null;
		Wikijump.modules.ForumViewThreadModule.vars.editActive = false;

		Wikijump.Editor.shutDown();

	},
	preview: function(e){

		var p = OZONE.utils.formToArray("p-new-post-form");
		OZONE.ajax.requestModule("forum/ForumPreviewPostModule", p, Wikijump.modules.ForumViewThreadModule.callbacks.preview);

	},
	closePreview: function(e){
		var previewContainer = document.getElementById("p-new-post-preview-div");
		previewContainer.style.display="none";
		OZONE.visuals.scrollTo("new-post-form-container");
	},
	save: function(e){
		var p = OZONE.utils.formToArray("p-new-post-form");
		p['action'] = "ForumAction";
		p['event'] = "savePost";
		if(Wikijump.modules.ForumViewThreadModule.vars.currentParentId){
			p['parent_post_id'] = Wikijump.modules.ForumViewThreadModule.vars.currentParentId;
		}
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.ForumViewThreadModule.callbacks.save);
	},

	togglePostOptions: function(event,postId){
		var oDiv = $("fpoptions-"+postId);
		if(oDiv.style.display != "block"){
			var inner = $("post-options").innerHTML;
			inner = inner.replace(/%POST_ID%/, postId);
			oDiv.innerHTML = inner;

			// modify permalink...
			var els = oDiv.getElementsByTagName('a');
			for(var i=0; i<els.length; i++){
				if(els[i].innerHTML == 'permalink'){
					els[i].href=document.getElementById("fpermalink-template").innerHTML+postId;
				}

			}
			var ofx = new fx.Opacity(oDiv.id,{duration:200});
			ofx.setOpacity(0);
			oDiv.style.display = "block";
			ofx.custom(0,1);
		} else {
			var ofx = new fx.Opacity(oDiv.id,{duration:200});
			ofx.custom(1,0);
			setTimeout('document.getElementById("fpoptions-'+postId+'").style.display="none"', 300);
		}
	},
	togglePostFold: function(event,postId){
		fDiv = $("post-"+postId); // leave it global... nasty.

		var ofx = new fx.Opacity(fDiv,{duration: 200, onComplete: function(){
			if(fDiv.className.indexOf(' post-folded')>=0){
				fDiv.className = fDiv.className.replace(/ post\-folded/,'');
			} else {
				fDiv.className += ' post-folded';
			}

			var ofx = new fx.Opacity(fDiv,{duration: 200});
			ofx.setOpacity(0);
			ofx.custom(0,1)
		}});
		ofx.custom(1,0);

	},
	permanentLink: function(postId){
		var t2 = new OZONE.dialogs.SmallInfoBox();
		t2.content='<h1>Permanent link</h1><p>Permanent link for this post is:</p>' +
				'<p><strong>'+HTTP_SCHEMA+"://"+WIKIREQUEST.info.domain+'/forum:thread/t/'+Wikijump.forumThreadId+'#post-'+postId+'</strong></p>'	;
		t2.buttons = ["close"];
		t2.title=" ";
		t2.clickOutsideToClose = true;

		t2.addButtonListener("close", t2.close);
		t2.show();
	},
	goToActiveEdit: function(e){
		if(Wikijump.modules.ForumViewThreadModule.vars.editActive){
			OZONE.dialog.cleanAll();
			OZONE.visuals.scrollTo('new-post-form-container');
		}
	},
	editClick: function(e, postId){
		// check if edit is not active somewhere
		if(Wikijump.Editor.editElementId){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content="You have an active editor somewhere already and it is not" +
					" possible to edit multiple elemnts at once.<br/><br/>" +
					'(<a href="javascript:;" onclick="OZONE.visuals.scrollTo(\''+Wikijump.Editor.editElementId+'\');OZONE.dialog.cleanAll()">scroll to active editor</a>)';
			w.show();
			return;
		}
		// ok, insert edit box below the edited post
		Wikijump.modules.ForumViewThreadModule.vars.currentEditPostId = postId;
	}
}

Wikijump.modules.ForumViewThreadModule.callbacks = {

	newPost: function(r){

	},
	preview: function(r){
		var previewContainer = document.getElementById("p-new-post-preview-div");
		var divNum;
		if(Wikijump.modules.ForumViewThreadModule.vars.fakedParent){
			divNum = 1;
		}else{
			divNum = 0;
		}
		previewContainer.getElementsByTagName('div').item(divNum).innerHTML=r.body;
		previewContainer.style.visibility="hidden";
		previewContainer.style.display="block";

		// a trick. scroll first FAST and...
		previewContainer.style.visibility="visible";
		OZONE.visuals.scrollTo("p-new-post-preview-div");
	},
	save: function(r){

		//if successfull - reload page?
		window.location.hash ="post-"+r.postId;

		window.location.reload();
		setTimeout('alert("reloaded")', 1000);
	}

}

Wikijump.modules.ForumViewThreadModule.init = function(){
	OZONE.utils.formatDates();
}

Wikijump.modules.ForumViewThreadModule.init();
