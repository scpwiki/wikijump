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

WIKIDOT.modules.ManageSiteAnonymousAbuseModule = {};
WIKIDOT.modules.ManageSiteAnonymousAbuseModule.vars = {};

WIKIDOT.modules.ManageSiteAnonymousAbuseModule.listeners = {
	clear: function(e, address, proxy){
		var p = new Object();
		p.action = "ManageSiteAbuseAction";
		p.event = "clearAnonymousFlags";
		p.address = address;
		if(proxy == 'proxy'){
			p.proxy="yes";
		}
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteAnonymousAbuseModule.callbacks.clear);
	},
	blockIp: function(e, address){
		
		WIKIDOT.modules.ManageSiteAnonymousAbuseModule.vars.currentIP = address;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("ban-ip-dialog").innerHTML.replace(/%%IP%%/, address);
		w.buttons = ['cancel', 'yes, ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, ban', WIKIDOT.modules.ManageSiteAnonymousAbuseModule.listeners.blockIp2);
		w.show();
	},
	blockIp2: function(e){
		var p = new Object();
		p.ips = WIKIDOT.modules.ManageSiteAnonymousAbuseModule.vars.currentIP;
		
		p.action = "ManageSiteBlockAction";
		p.event = "blockIp";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteAnonymousAbuseModule.callbacks.blockIp);
	}
	
}

WIKIDOT.modules.ManageSiteAnonymousAbuseModule.callbacks = {
	clear: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Flags cleared";
		w.show();
		
	},
	blockIp: function(r){
		if(r.status !== 'ok'){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = r.message;
			if(r.errormess){
				w.content += '<br/>'+r.errormess;
			}
			w.show();
			return;
		}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "The IP address added to the block list.";
		w.show();
		
	}
}
