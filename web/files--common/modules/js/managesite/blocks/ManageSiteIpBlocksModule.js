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

WIKIDOT.modules.ManageSiteIpBlocksModule = {};

WIKIDOT.modules.ManageSiteIpBlocksModule.vars = {
	addFormInited: false,
	currentIp: null,
	dCurrentBlockId: null
}

WIKIDOT.modules.ManageSiteIpBlocksModule.listeners = {
	showAddForm: function(e){
		if(!WIKIDOT.modules.ManageSiteIpBlocksModule.vars.addFormInited){
			var limiter = new OZONE.forms.lengthLimiter("block-reason", "reason-char-left", 200);
			WIKIDOT.modules.ManageSiteIpBlocksModule.vars.addFormInited = true;
		}
		$("show-add-block-button").style.display = "none";
		$("add-block-div").style.display = "block";
		OZONE.visuals.scrollTo("add-block-div");
	},
	
	cancelAdd: function(e){
		// resets the forms?
		$("show-add-block-button").style.display = "block";
		$("add-block-div").style.display = "none";
		$("ip-errors").style.display = "none";
	},
	
	blockIp: function(e){
		var p = new Object();
		p.ips = $("block-ips").value;
		if(p.ips == ''){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "IP address(es) field is blank.";
			w.show();
			return;
		}
		p.reason = $("block-reason").value;
		p.action = "ManageSiteBlockAction";
		p.event = "blockIp";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteIpBlocksModule.callbacks.blockIp);
	},
	
	deleteBlock: function(e, blockId, ip){
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.buttons = ['cancel', 'yes, delete block'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, delete block', WIKIDOT.modules.ManageSiteIpBlocksModule.listeners.deleteBlock2, ip);
		w.content = "Are you sure you want to remove the block for the IP <strong>"+ip+"</strong>?";
		w.show();
		WIKIDOT.modules.ManageSiteIpBlocksModule.vars.dCurrentBlockId = blockId;
		
	},
	deleteBlock2: function(e){
		var blockId = WIKIDOT.modules.ManageSiteIpBlocksModule.vars.dCurrentBlockId;
		var p = new Object();
		p.blockId = blockId;
		p.action = "ManageSiteBlockAction";
		p.event = "deleteIpBlock";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteIpBlocksModule.callbacks.deleteBlock);
	}
}

WIKIDOT.modules.ManageSiteIpBlocksModule.callbacks = {
	blockIp: function(r){
		if(r.status == 'ip_errors'){
			var errors = r.errorIps;
			$("ip-errors").innerHTML = r.errormess;
			$("ip-errors").style.display = "block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "IP(s) added to the block list.";
		w.show();
		// refresh the screen too
		setTimeout('WIKIDOT.modules.ManagerSiteModule.utils.loadModule("sm-ip-blocks")', 1500);
		
	},
	deleteBlock: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "IP block removed.";
		w.show();
		// refresh the screen too
		setTimeout('WIKIDOT.modules.ManagerSiteModule.utils.loadModule("sm-ip-blocks")', 1500);
		
	}
}

WIKIDOT.modules.ManageSiteIpBlocksModule.init = function(){
	
}

WIKIDOT.modules.ManageSiteIpBlocksModule.init();
