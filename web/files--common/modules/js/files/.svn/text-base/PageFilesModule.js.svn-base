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

WIKIDOT.modules.PageFilesModule = {};

WIKIDOT.modules.PageFilesModule.vars = {
	fileIdOptions: null, // file id for which options are active.
	fileName: null
}

WIKIDOT.modules.PageFilesModule.listeners = {
	
	fileManager: function(e){
		OZONE.ajax.requestModule("files/manager/FileManagerModule",{pageId:WIKIREQUEST.info.pageId} ,WIKIDOT.modules.PageFilesModule.callbacks.showUploadClick);
	},
	
	showUploadClick: function(e){
		OZONE.ajax.requestModule("files/FileUploadModule",{pageId:WIKIREQUEST.info.pageId} ,WIKIDOT.modules.PageFilesModule.callbacks.showUploadClick);
	},
	
	fileMoreInfo: function(e, fileId){
		if(fileId == null){return;}
		
		var p = new Object();
		p['file_id'] = fileId;
		OZONE.ajax.requestModule("files/FileInformationWinModule", p, WIKIDOT.modules.PageFilesModule.callbacks.fileMoreInfo);
		
	},
	renameFile: function(e){
		var fileId = WIKIDOT.modules.PageFilesModule.vars.fileIdOptions;
		if(fileId == null){return;}
		var p = new Object();
		p['file_id'] = fileId;
		OZONE.ajax.requestModule("files/FileRenameWinModule", p, WIKIDOT.modules.PageFilesModule.callbacks.renameFile);
		
	},
	renameFile2: function(e, force){
		var fileId = WIKIDOT.modules.PageFilesModule.vars.fileIdOptions;
		if(fileId == null){return;}
		var p = new Object();
		p['file_id'] = fileId;
		p['new_name'] = $("file-rename-name").value;
		p['action'] = "FileAction";
		p['event'] = 'renameFile';
		if(force == true){
			p['force'] = true;
		}
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.PageFilesModule.callbacks.renameFile2);
		
	},
	moveFile: function(e){
		var fileId = WIKIDOT.modules.PageFilesModule.vars.fileIdOptions;
		if(fileId == null){return;}
		var p = new Object();
		p['file_id'] = fileId;
		OZONE.ajax.requestModule("files/FileMoveWinModule", p, WIKIDOT.modules.PageFilesModule.callbacks.moveFile);
		
	},
	moveFile2: function(e, force){
		var fileId = WIKIDOT.modules.PageFilesModule.vars.fileIdOptions;
		if(fileId == null){return;}
		var p = new Object();
		p['file_id'] = fileId;
		p['destination_page_name'] = $("file-move-page").value;
		p['action'] = "FileAction";
		p['event'] = 'moveFile';
		if(force == true){
			p['force'] = true;
		}
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.PageFilesModule.callbacks.moveFile2);
	},
	
	deleteFile: function(e){
		var fileName = $("file-row-"+WIKIDOT.modules.PageFilesModule.vars.fileIdOptions).getElementsByTagName("td")[0].getElementsByTagName("a")[0].innerHTML;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = "<h1>Delete file <em>"+fileName+"</em></h1> Are you sure you want to delete this file?";
		w.buttons = ['no, cancel', 'yes, delete'];
		w.addButtonListener('no, cancel', w.close);
		w.addButtonListener('yes, delete', WIKIDOT.modules.PageFilesModule.listeners.deleteFile2);
		w.show();
	},
	deleteFile2: function(e){
		var fileId = WIKIDOT.modules.PageFilesModule.vars.fileIdOptions;
		if(fileId == null){return;}
		var p = new Object();
		p['file_id'] = fileId;
		p['action'] = "FileAction";
		p['event'] = 'deleteFile';
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.PageFilesModule.callbacks.deleteFile2);
		
	}
}

WIKIDOT.modules.PageFilesModule.callbacks = {
	showUploadClick: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent('file-action-area', r.body);
		setTimeout("OZONE.visuals.scrollTo('file-action-area')", 100);
	}	,
	fileMoreInfo: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var t2 = new OZONE.dialogs.Dialog();
		t2.title = "File information";
		t2.content=r.body	;
		t2.buttons = ["close"];
		t2.style.width = "60em";
		t2.clickOutsideToClose = true;
		t2.focusButton = 'close';
		
		t2.addButtonListener("close", t2.close);
		t2.show();
	},
	renameFile: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var t2 = new OZONE.dialogs.Dialog();
		t2.title = "Rename file";
		t2.content=r.body	;
		t2.buttons = ["cancel", "rename"];
		
		t2.addButtonListener("cancel", t2.close);
		t2.addButtonListener("rename", WIKIDOT.modules.PageFilesModule.listeners.renameFile2);
		t2.show();
		
	},
	
	renameFile2: function(r){
		if(r.status == 'ok'){
			var t2 = new OZONE.dialogs.SuccessBox();
			t2.content="The file has been renamed!"	;
			t2.show();
			setTimeout('OZONE.dialog.cleanAll();WIKIDOT.page.listeners.filesClick(null)', 1000);
		}else if(r.status == "file_exists"){
			var t2 = new OZONE.dialogs.Dialog();
			t2.title = "Rename file";
			t2.content = r.body;
			t2.show();
		}else if(r.status == 'name_error'){
			$("rename-error-block").style.display="block";
			$("rename-error-block").innerHTML=r.message;
		}else{
			if(!WIKIDOT.utils.handleError(r)) {return;}
			
		}
	},
	moveFile: function(r){
		if(r.status == 'ok'){
			var t2 = new OZONE.dialogs.Dialog();
			t2.content=r.body	;
			t2.buttons = ["cancel", "move"];
			t2.title="Move file";
			
			t2.addButtonListener("cancel", t2.close);
			t2.addButtonListener("move", WIKIDOT.modules.PageFilesModule.listeners.moveFile2);
			t2.show();
			
			// attach the autocomplete thing
			var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
			myDataSource.scriptQueryParam="q";
			myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";
	
			var myAutoComp = new YAHOO.widget.AutoComplete("file-move-page","file-move-page-autocomplete", myDataSource);
			myAutoComp.formatResult = function(aResultItem, sQuery) { 
				var title = aResultItem[1];
				var unixName = aResultItem[0];
				if(unixName!= null){
					return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 85%;">('+title+')</div>';
				} else {
					return "";
				}
		
			}
			myAutoComp.minQueryLength = 2;
			myAutoComp.queryDelay = 0.5;
			myAutoComp.forceSelection = false;
		}else{
			if(!WIKIDOT.utils.handleError(r)) {return;}
		}
	},
	
	moveFile2: function(r){
		if(r.status == 'ok'){
			var t2 = new OZONE.dialogs.SuccessBox();
			t2.content="The file has been moved!"	;
			t2.show();
			setTimeout('OZONE.dialog.cleanAll();WIKIDOT.page.listeners.filesClick(null)', 1000);
		}else if(r.status == "file_exists"){
			var t2 = new OZONE.dialogs.Dialog();
			t2.title = "Move file";
			t2.content = r.body;
			t2.show();
		}else if(r.status == 'no_destination' || r.status == 'no_destination_permission'){
			$("file-move-error").innerHTML = r.message;
			$("file-move-error").style.display ="block";
		} else {
			if(!WIKIDOT.utils.handleError(r)) {return;}
		}
	},
	deleteFile2: function(r){
		if(r.status == 'ok'){
			var t2 = new OZONE.dialogs.SuccessBox();
			t2.content="The file has beed deleted!";
			t2.show();
			setTimeout('OZONE.dialog.cleanAll();WIKIDOT.page.listeners.filesClick(null)', 1000);
		}else{
			if(!WIKIDOT.utils.handleError(r)) {return;}
		}
	}
}

toggleFileOptions = function(fileId){
	var trid = "file-row-"+fileId;
	var trido = "file-options-row-"+fileId;
	var t = $(trido);
	if(t){
		$(trid).className="";
		var eff = new fx.Opacity(t, {duration: 200});
		eff.custom(1,0);
		setTimeout('$("'+trido+'").parentNode.removeChild($("'+trido+'"))', 300);
		WIKIDOT.modules.PageFilesModule.vars.fileIdOptions = null;

	}else {
		
		if(WIKIDOT.modules.PageFilesModule.vars.fileIdOptions && WIKIDOT.modules.PageFilesModule.vars.fileIdOptions != fileId){
			toggleFileOptions(WIKIDOT.modules.PageFilesModule.vars.fileIdOptions);
		}
		
		var optionsContent = $("file-options-template").innerHTML;
		// add new row.
		var row = document.createElement('tr');
		var td = document.createElement('td');
		td.colSpan = 4;
		row.id=trido;
		row.appendChild(td);
		td.className = "options";
		td.innerHTML = optionsContent;
		var trf = $(trid);
		trf.className="highlight";
		row.className="highlight";
		var eff = new fx.Opacity(row, {duration: 200});
		eff.setOpacity(0);
		OZONE.dom.insertAfter(trf.parentNode, row, trf);
		eff.custom(0,1);
		WIKIDOT.modules.PageFilesModule.vars.fileIdOptions = fileId;
	}

}

YAHOO.util.Event.addListener("show-upload-button", "click", WIKIDOT.modules.PageFilesModule.listeners.showUploadClick);
WIKIDOT.utils.formatDates("action-area");
