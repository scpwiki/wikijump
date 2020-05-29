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

WIKIDOT.modules.PageUploadModule = {}

WIKIDOT.modules.PageUploadModule.listeners = {
	fileUploaded: function(fileStatus, message){
		if(fileStatus == "ok"){
			var t2 = new OZONE.dialogs.SuccessBox();
			t2.content="File succesfully uploaded!"	;
			t2.show();
			setTimeout('OZONE.dialog.cleanAll();WIKIDOT.page.listeners.filesClick(null)', 2000);
		}else{
			var t3 = new OZONE.dialogs.ErrorDialog();
			if(!message || message ==''){
				message = "Error uploading file";
			}
			t3.content=message;
			t3.show();
		}
		$('_upload_iframe').src='/common--misc/blank.html';
	},
	
	uploadStart: function(e){
		var t2 = new OZONE.dialogs.WaitBox();
		t2.content='Uploading file... Please wait...';
		t2.show();
	},
	uploadCancel: function(e){
		$("file-action-area").innerHTML="";
		YAHOO.util.Event.stopEvent(e);
	},
	
	checkFileExists: function(e){
		if($("upload-userfile").value == ''){
			return;
		}
		var p = new Object();
		if($("upload-dfilename").value != ''){
			p.filename = $("upload-dfilename").value;
		} else{
			p.filename = $("upload-userfile").value;
		}
		
		p.action = 'FileAction';
		p.event = 'checkFileExists';
		p.pageId = WIKIREQUEST.info.pageId;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PageUploadModule.callbacks.checkFileExists);
		YAHOO.util.Event.stopEvent(e);
	},
	
	forceOverwrite: function(e){
		var i = document.createElement('input');
		i.type='hidden';
		i.name='force';
		i.value='true';
		$("file-upload-form").appendChild(i);
		$("file-upload-form").submit();
		WIKIDOT.modules.PageUploadModule.listeners.uploadStart(null);
	}
}

WIKIDOT.modules.PageUploadModule.callbacks = {
	checkFileExists: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		if(r.exists == true){
			var w = new OZONE.dialogs.Dialog();
			w.title = "File exists";
			w.content = r.body;
			w.show();
		}else{
			// just submit the form now!
			$("file-upload-form").submit();
			WIKIDOT.modules.PageUploadModule.listeners.uploadStart(null);
			
		}
	}
}

function test(text){
	alert(text);
}
WIKIDOT.modules.PageUploadModule.init = function(){
	$("file-upload-form-page-id").value=WIKIREQUEST.info.pageId;
	YAHOO.util.Event.addListener("file-upload-form", "submit", WIKIDOT.modules.PageUploadModule.listeners.uploadStart);
	var limiter = new OZONE.forms.lengthLimiter("file-comments", "file-comments-charleft", 100);
	
}

WIKIDOT.modules.PageUploadModule.init();
