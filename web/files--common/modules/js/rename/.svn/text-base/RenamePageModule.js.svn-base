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

WIKIDOT.modules.RenamePageModule = {};

WIKIDOT.modules.RenamePageModule.vars = {};

WIKIDOT.modules.RenamePageModule.listeners = {
	rename: function(e){
		var p = new Object();
		p['action'] = 'WikiPageAction';
		p['event'] = 'renamePage';
		p['page_id'] =  WIKIREQUEST.info.pageId;
		p['new_name'] = $("move-new-page-name").value;
		
		// resolve any pages to fix deps
		var inps = $("rename-backlinks-box").getElementsByTagName('input');
		var fixdeps = new Array();
		for(var i=0; i<inps.length; i++){
			if(inps[i].checked){
				fixdeps.push(inps[i].id.replace(/rename\-dep\-fix\-/, ''));
			}
		}
		if(fixdeps.length>0){
			p['fixdeps'] = fixdeps.join(',');
		}
		
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.RenamePageModule.callbacks.rename);
		
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
		
		// resolve any pages to fix deps
		var inps = $("rename-backlinks-box").getElementsByTagName('input');
		var fixdeps = new Array();
		for(var i=0; i<inps.length; i++){
			if(inps[i].checked){
				fixdeps.push(inps[i].id.replace(/rename\-dep\-fix\-/, ''));
			}
		}
		if(fixdeps.length>0){
			p['fixdeps'] = fixdeps.join(',');
		}
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Renaming/moving page...";
		w.show();
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.RenamePageModule.callbacks.rename);
	},
	showBacklinks: function(e){
		var pageId = WIKIREQUEST.info.pageId;
		var parms = new Object();
		parms['page_id'] = pageId;
		OZONE.ajax.requestModule("rename/RenameBacklinksModule",parms,WIKIDOT.modules.RenamePageModule.callbacks.showBacklinks);
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
		
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.RenamePageModule.callbacks.deletePage);
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Deleting page...";
		w.show();
	}
}

WIKIDOT.modules.RenamePageModule.callbacks = {
	rename: function(r){
		if(r.status == "page_exists" || r.status == 'not_allowed' || r.status == "no_new_name"){
			$("rename-error-block").style.display="block";
			$("rename-error-block").innerHTML = r.message;
			OZONE.dialog.cleanAll();
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		if(r.locks){
			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.show();
			return;
		}
		
		if(r.leftDeps){
			WIKIDOT.modules.RenamePageModule.vars.newName = r.newName;
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
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent('rename-backlinks-box', r.body);
		$("rename-backlinks-box").style.display = "block";
		$("rename-show-backlinks").style.display="none";
		$("rename-hide-backlinks").style.display="";
	},
	
	deletePage: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var t2 = new OZONE.dialogs.SuccessBox();
		t2.content="The page has been deleted!";
		t2.show();
		setTimeout('window.location.reload()',1500);
	}
		
}
