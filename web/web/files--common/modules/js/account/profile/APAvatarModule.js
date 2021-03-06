

Wikijump.modules.APAvatarModule = {};

Wikijump.modules.APAvatarModule.vars = {
	im48: null,
	im16: null
}

Wikijump.modules.APAvatarModule.listeners = {

	startUpload: function(e){
		$("upload-wait").style.display = "block";
	},

	uploaded: function(status, im48, im16){
		if(status != "ok"){
			var er = new OZONE.dialogs.ErrorDialog();
			er.content = "The uploaded file cannot be used as a buddy icon.<br/>" +
					"Please upload a valid .png, .jpg or .gif image.";
			er.show();
//			alert("The uploaded file cannot be used as a buddy icon.\n" +
			return;
		}
		var path = '/common--tmp/avatars-upload/';
		$("avatar-preview-large").src = path+im48;
		$("avatar-preview-small").src = path+im16;

		Wikijump.modules.APAvatarModule.vars.im16 = im16;
		Wikijump.modules.APAvatarModule.vars.im48 = im48;
		$("avatar-preview").style.display="block";
		$('file-upload-div').style.display='none';
		$("uri-upload-div").style.display='none';
		$("upload-wait").style.display = "none";
	},

	useIt: function(e){
		// sets the avatar permanently
		if(!Wikijump.modules.APAvatarModule.vars.im16){return;}
		var p = new Object();
		p['im48'] = Wikijump.modules.APAvatarModule.vars.im48;
		p['im16'] = Wikijump.modules.APAvatarModule.vars.im16;
		p['action'] = 'AccountProfileAction';
		p['event'] = "setAvatar";
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.APAvatarModule.callbacks.useIt);
	},
	reset: function(e){
		$('avatar-choice1').style.display='';
		$('file-upload-div').style.display='none';
		$("uri-upload-div").style.display='none';
		$("avatar-preview").style.display='none';
		$("upload-wait").style.display = "none";
		Wikijump.modules.APAvatarModule.vars.im16 = null;
		Wikijump.modules.APAvatarModule.vars.im48 = null;
		YAHOO.util.Event.stopEvent(e);

	},
	deleteAvatar: function(e){
		var p = new Object();
		p['action'] = 'AccountProfileAction';
		p['event'] = "deleteAvatar";
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.APAvatarModule.callbacks.deleteAvatar);
	},

	uploadUri: function(e){
		var uri = $("upload-uri").value;
		if(!uri.match(/^(http[s]?:\/\/)|(ftp:\/\/)[a-zA-Z0-9\-]+\/.*/)){
			var er = new OZONE.dialogs.ErrorDialog();
			er.content = "This is not a valid URI address.";
			er.show();
			return;
		}
		$("upload-wait").style.display = "block";
		var p = new Object();
		p['action'] = 'AccountProfileAction';
		p['event'] = "uploadAvatarUri";
		p['uri'] = uri;
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.APAvatarModule.callbacks.uploadUri);

	}
}

Wikijump.modules.APAvatarModule.callbacks = {
	useIt: function(r){
		/*
		$('file-upload-div').style.display='none';
		$("uri-upload-div").style.display='none';
		$("avatar-preview").style.display='none';
		$("avatar-success").style.display='block';
		Wikijump.modules.APAvatarModule.vars.im16 = null;
		Wikijump.modules.APAvatarModule.vars.im48 = null;
		*/
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your buddy icon has been changed!";
		w.show();
		setTimeout('OZONE.ajax.requestModule("account/profile/APAvatarModule", null, Wikijump.modules.AccountModule.callbacks.menuClick)', 1500);

	},
	deleteAvatar: function(r){
		// simply reload this module.
		OZONE.ajax.requestModule('account/profile/APAvatarModule', null, Wikijump.modules.AccountModule.callbacks.menuClick)
	},
	uploadUri: function(r){
		if(r.status != "ok"){
			var er = new OZONE.dialogs.ErrorDialog();
			er.content = "This image cannot be used as your buddy icon. ("+r.status+")";
			er.show();
			return;
		}
		Wikijump.modules.APAvatarModule.listeners.uploaded(r.status, r.im48,r.im16);

	}
}

Wikijump.modules.APAvatarModule.init = function(){

}
Wikijump.modules.APAvatarModule.init();
