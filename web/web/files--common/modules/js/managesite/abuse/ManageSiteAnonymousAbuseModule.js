

Wikijump.modules.ManageSiteAnonymousAbuseModule = {};
Wikijump.modules.ManageSiteAnonymousAbuseModule.vars = {};

Wikijump.modules.ManageSiteAnonymousAbuseModule.listeners = {
	clear: function(e, address, proxy){
		var p = new Object();
		p.action = "ManageSiteAbuseAction";
		p.event = "clearAnonymousFlags";
		p.address = address;
		if(proxy == 'proxy'){
			p.proxy="yes";
		}
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteAnonymousAbuseModule.callbacks.clear);
	},
	blockIp: function(e, address){

		Wikijump.modules.ManageSiteAnonymousAbuseModule.vars.currentIP = address;
		var w = new OZONE.dialogs.ConfirmationDialog();
		w.content = $("ban-ip-dialog").innerHTML.replace(/%%IP%%/, address);
		w.buttons = ['cancel', 'yes, ban'];
		w.addButtonListener('cancel', w.close);
		w.addButtonListener('yes, ban', Wikijump.modules.ManageSiteAnonymousAbuseModule.listeners.blockIp2);
		w.show();
	},
	blockIp2: function(e){
		var p = new Object();
		p.ips = Wikijump.modules.ManageSiteAnonymousAbuseModule.vars.currentIP;

		p.action = "ManageSiteBlockAction";
		p.event = "blockIp";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ManageSiteAnonymousAbuseModule.callbacks.blockIp);
	}

}

Wikijump.modules.ManageSiteAnonymousAbuseModule.callbacks = {
	clear: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
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
