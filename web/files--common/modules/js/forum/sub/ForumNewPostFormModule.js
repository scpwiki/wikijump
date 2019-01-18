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

WIKIDOT.modules.ForumNewPostFormModule = {};

WIKIDOT.modules.ForumNewPostFormModule.listeners = {
	preview: function(e){
		var p = OZONE.utils.formToArray("new-post-form");	
		OZONE.ajax.requestModule("forum/ForumPreviewPostModule", p, WIKIDOT.modules.ForumNewPostFormModule.callbacks.preview);
	},
	cancel: function(e){
		// remove form
		var formDiv = $('new-post-form-container');
		formDiv.parentNode.removeChild(formDiv);
		$("new-post-button").style.display="";

		WIKIDOT.Editor.shutDown();
	},
	closePreview: function(e){
		$("new-post-preview-div").style.display = "none";
	},
	save: function(e){
		var p = OZONE.utils.formToArray("new-post-form");	
		p.action = "ForumAction";
		p.event = "savePost";
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Posting now...";
		w.show();
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ForumNewPostFormModule.callbacks.save);
		
	}
}

WIKIDOT.modules.ForumNewPostFormModule.callbacks = {
	preview: function(r){

		if(!WIKIDOT.utils.handleError(r)) {return;}	
			
		var previewContainer = $("new-post-preview-div");

		var ctmp = previewContainer.getElementsByTagName('div');
		ctmp[0].innerHTML = r.body;
		previewContainer.style.visibility="hidden";
		previewContainer.style.display="block";
				
		// a trick. scroll first FAST and...
		previewContainer.style.visibility="visible";
		OZONE.visuals.scrollTo("new-post-preview-div");	
		
	},
	save: function(r){
		if(r.status=="form_errors"){
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";
			
			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}		
			inner += "</ul>";
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = inner;
			w.show();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your post has been saved.";
		w.show();
		var hash = "post-"+r.postId;
		// new uri:
		var uri = window.location.href.replace(/#.*$/, '');
		uri = uri.replace(/\/$/,'');
		if(!WIKIREQUEST.info.requestPageName.match(/^forum:thread$/)){
			if(!uri.match(/comments\/show/)){
				uri += '/comments/show';
				uri += "#post-"+r.postId;
				setTimeout('window.location.href="'+uri+'"', 1000);
			}else{
				var hash="post-"+r.postId;
				setTimeout('window.location.hash = "'+hash+'"; window.location.reload()', 1000);
			}
			
		}else{
			var hash="post-"+r.postId;
			setTimeout('window.location.hash = "'+hash+'"; window.location.reload()', 1000);
		}

	}
}
