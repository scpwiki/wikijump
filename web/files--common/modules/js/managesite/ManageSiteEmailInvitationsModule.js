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

WIKIDOT.modules.ManageSiteEmailInvitationsModule = {};

WIKIDOT.modules.ManageSiteEmailInvitationsModule.vars = {};

WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners = {
	moreRecipients: function(e, rname, email){
		if(false && $("invitation-addresses").getElementsByTagName("table").length > 200){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "Sorry, you can not send more than 200 invitations at once.";
			w.show();
			return;
		}
		var temp = $("recipient-template").getElementsByTagName('table')[0];
		var clone = temp.cloneNode(true);
		if(rname || email){
			var inpts = clone.getElementsByTagName('input');
			inpts[0].value = rname;
			inpts[1].value = email;
		}
		$("invitation-addresses").appendChild(clone);
	},
	
	removeRecipient: function(e){
		// get the parrent "table" element
		var el = YAHOO.util.Event.getTarget(e);
		while(el && el.tagName.toLowerCase() != "table"){
			el = el.parentNode;	
		}
		if(el){
			el.parentNode.removeChild(el);
		}
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.updateTo();
	},

	updateTo: function(e){
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.utils.updateAddresses(null);
		var adrs = WIKIDOT.modules.ManageSiteEmailInvitationsModule.vars.addresses;
		if(!adrs) {return;}
		var frmt = new Array();

		for(var i=0; i<adrs.length; i++){
			frmt.push(adrs[i][1]+' <' + adrs[i][0]+'>');
		}
		
		$("recipients-list-formatted").innerHTML = OZONE.utils.escapeHtml(frmt.join(', '));
	},
	
	send: function(e){
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.utils.updateAddresses(null);
		var adrs = WIKIDOT.modules.ManageSiteEmailInvitationsModule.vars.addresses;
		
		if(adrs.length==0){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "No valid recepients have been given. For each person both the email address and his/her name should be given.";
			w.show();
			return;
		
		}
		
		// check for invalid records
		var invalids = YAHOO.util.Dom.getElementsByClassName('invalid', 'input',$("invitation-addresses"));
		if(invalids.length>0){
			if(!confirm("The list contains incomplete entries. Are you sure you want to continue? \nOnly valid entries will be used to send invitations if you continue.")){
				return;
			}
		}
		// make a confirmation? no.

		var serialized = JSON.stringify(adrs);
		var p= new Object();
		p['addresses'] = serialized;
		p.event = 'sendEmailInvitations';
		p.action = 'ManageSiteMembershipAction';
		p.message = $("inv-message").value;
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteEmailInvitationsModule.callbacks.send);
			
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Sending invitations...";
		w.show();
	},
	showBulkAdd: function(e){
		$("invitation-addresses-upload-box").style.display = "none";
		$("invitation-addresses-bulk-box").style.display = "block";
		OZONE.visuals.scrollTo($("invitation-addresses-bulk-box"));
		
	},
	cancelBulkAdd: function(e){
		$("invitation-addresses-bulk-box").style.display = "none";
	},
	
	processBulkAdd: function(e){
		var text = $("invitation-addresses-bulk-text").value;
		var entries = text.split(/[\n,]+/);
		var en2 = new Array();
		var email, rname;
		var eReg = new RegExp("[a-z0-9\._\-]+@[a-z0-9\-]+(\.[a-z0-9\-]+)+", "i");
		for(var i=0; i<entries.length; i++){
			if(eReg.test(entries[i])){
				email = eReg.exec(entries[i])[0];
				rname = entries[i].replace(eReg, '');
				rname = rname.replace(/[<>]/g, ' ');
				rname = rname.replace(/ +/g, ' ');
				rname = rname.replace(/^ +/, ' ');
				rname = rname.replace(/ +$/, ' ');
				WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients(null, rname, email);
			}
		}
		$("invitation-addresses-bulk-box").style.display = "none";
		$("invitation-addresses-bulk-text").value = '';
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.tidyList(null);
	},
	
	showUpload: function(e){
		$("invitation-addresses-bulk-box").style.display = "none";
		$("invitation-addresses-upload-box").style.display = "block";
		OZONE.visuals.scrollTo($("invitation-addresses-upload-box"));
	},
	cancelUpload: function(e){
		$("invitation-addresses-upload-box").style.display = "none";
		
	},
	setAllToContacts: function(e, value){
		var tbls = $("invitation-addresses").getElementsByTagName("table");
		for(var i=0; i< tbls.length; i++){
			var inpts = tbls[i].getElementsByTagName('input');
			inpts[2].checked = value;
		}
	},
	
	tidyList: function(e){
		// remove empty elements, remove duplicates, add a few empty at the end
		var tbls = $("invitation-addresses").getElementsByTagName("table");
		var emails = new Array();
		var toRemove = new Array();
		for(var i=0; i< tbls.length; i++){
			// get values
			var inpts = tbls[i].getElementsByTagName('input');
			email = inpts[1].value;
			if(email == ''){
				toRemove.push(tbls[i]);
			}
			// check if email already in the emails
			for(var j=0; j<emails.length; j++){
				if(email == emails[j]){
					toRemove.push(tbls[i]);
					break;
				}
			}
			emails.push(email);
		}
		
		for(var i=0; i<toRemove.length; i++){
			if(toRemove[i].parentNode){
				toRemove[i].parentNode.removeChild(toRemove[i]);
			}
		}
		
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.updateTo(null);
		
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients();
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients();
	},
	
	startUpload: function(e){},
	
	contactsUploaded: function(status, addr){
		if(status != "ok"){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "Error uploading the file.";
			w.show();
			return;
		}
		
		var ads = JSON.parse(addr);
		
		for(var i=0;i<ads.length; i++){
			WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients(null, ads[i]['name'], ads[i]['email']);
		}
		
		$("invitation-addresses-upload-box").style.display = "none";
		
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.tidyList(null);
		
	}
}

WIKIDOT.modules.ManageSiteEmailInvitationsModule.callbacks = {
	send: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Invitations have been saved";
		w.show();
		
		window.setTimeout("WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-invitations-history');OZONE.visuals.scrollTo('header');", 1000);
		
	}	

}

WIKIDOT.modules.ManageSiteEmailInvitationsModule.utils = {
	updateAddresses: function(e){
		// manually create a list of addresses
		var adrs = new Array();
		var tbls = $("invitation-addresses").getElementsByTagName("table");
		for(var i=0; i< tbls.length; i++){
			// get values
			var inpts = tbls[i].getElementsByTagName('input');
			var email = inpts[1].value;
			if(email != '' && !email.match(/^[a-z0-9\._\-]+@[a-z0-9\-]+(\.[a-z0-9\-]+)+$/i)){
				YAHOO.util.Dom.addClass(inpts[1], 'invalid');
			}else{
				YAHOO.util.Dom.removeClass(inpts[1], 'invalid');
			}
			var rname = inpts[0].value;
			if(email!='' && email.match(/^[a-z0-9\._\-]+@[a-z0-9\-]+(\.[a-z0-9\-]+)+$/i) && rname != ''){
				adrs.push(new Array(email, rname, inpts[2].checked));
			}
			if(email != '' && rname == ''){
				YAHOO.util.Dom.addClass(inpts[0], 'invalid');
			}else{
				YAHOO.util.Dom.removeClass(inpts[0], 'invalid');
			}
		}
		WIKIDOT.modules.ManageSiteEmailInvitationsModule.vars.addresses = adrs;
	}
}

WIKIDOT.modules.ManageSiteEmailInvitationsModule.init = function(){
	WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients();
	WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients();
	WIKIDOT.modules.ManageSiteEmailInvitationsModule.listeners.moreRecipients();
}

WIKIDOT.modules.ManageSiteEmailInvitationsModule.init();
