

Wikijump.modules.RenamePageModule = {};

Wikijump.modules.RenamePageModule.vars = {};

Wikijump.modules.RenamePageModule.listeners = {
	rename: function(e){
		var p = new Object();
		p['action'] = 'WikiPageAction';
		p['event'] = 'renamePage';
		p['page_id'] =  WIKIREQUEST.info.pageId;
		p['new_name'] = $("move-new-page-name").value;

		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.RenamePageModule.callbacks.rename);

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Renaming/moving page...";
		w.show();
	},
	renameForce: function(e){
		var p = new Object();
		p['action'] = 'WikiPageAction';
		p['event'] = 'renamePage';
		p['page_id'] =  WIKIREQUEST.info.pageId;
		p['new_name'] = $("move-new-page-name").value;
		p['force'] = 'yes';

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Renaming/moving page...";
		w.show();
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.RenamePageModule.callbacks.rename);
	},
	showBacklinks: function(e){
		var pageId = WIKIREQUEST.info.pageId;
		var parms = new Object();
		parms['page_id'] = pageId;
		OZONE.ajax.requestModule("Rename/RenameBacklinksModule",parms,Wikijump.modules.RenamePageModule.callbacks.showBacklinks);
	},
	hideBacklinks: function(e){

		$("rename-show-backlinks").style.display="";
		$("rename-hide-backlinks").style.display="none";
		$('rename-backlinks-box').innerHTML="";
		$('rename-backlinks-box').style.display="none";
	},

	selectAll: function(e){
		var inps = $("rename-backlinks-box").getElementsByTagName('input');
		for(var i=0; i<inps.length; i++){
			inps[i].checked=true;
		}
	},
	unselectAll: function(e){
		var inps = $("rename-backlinks-box").getElementsByTagName('input');
		for(var i=0; i<inps.length; i++){
			inps[i].checked=false;
		}
	},

	deletePage: function(e){
		var p = new Object();
		p['action'] = 'WikiPageAction';
		p['event'] = 'deletePage';
		p['page_id'] =  WIKIREQUEST.info.pageId;

		var q = "Are you sure you want to completely wipe out this page?\n(Sorry, just wanted to make sure...)";

		if(!confirm(q)){
			return;
		}

		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.RenamePageModule.callbacks.deletePage);

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Deleting page...";
		w.show();
	}
}

Wikijump.modules.RenamePageModule.callbacks = {
	rename: function(r){
		if(r.status == "page_exists" || r.status == 'not_allowed' || r.status == "no_new_name"){
			$("rename-error-block").style.display="block";
			$("rename-error-block").innerHTML = r.message;
			OZONE.dialog.cleanAll();
			return;
		}
		if(!Wikijump.utils.handleError(r)) {return;}

		if(r.locks){
			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.show();
			return;
		}

		var t2 = new OZONE.dialogs.SuccessBox();
		t2.content="The page has been renamed!";
		t2.show();
		setTimeout('window.location.href=("/'+r.newName+'")',1500);

	},
	showBacklinks: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent('rename-backlinks-box', r.body);
		$("rename-backlinks-box").style.display = "block";
		$("rename-show-backlinks").style.display="none";
		$("rename-hide-backlinks").style.display="";
	},

	deletePage: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var t2 = new OZONE.dialogs.SuccessBox();
		t2.content="The page has been deleted!";
		t2.show();
		setTimeout('window.location.reload()',1500);
	}

}
