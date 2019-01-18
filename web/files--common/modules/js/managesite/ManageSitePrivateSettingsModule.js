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

WIKIDOT.modules.ManageSitePrivateSettingsModule = {};

WIKIDOT.modules.ManageSitePrivateSettingsModule.listeners ={
	save: function(e){
		var p = OZONE.utils.formToArray("sm-private-form");
		
		// get "viewers" list
		var sr = $("viewers-list-div");
		var ents = sr.getElementsByTagName('div');
		var uss = new Array();
		for(var i=0; i<ents.length;i++){
			var userId = ents[i].id.replace(/.*?([0-9]+)$/,"$1");
			uss.push(userId);
		}
		p.viewers = uss.join(',');
		p.action = "ManageSiteAction";
		p.event = "savePrivateSettings";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSitePrivateSettingsModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
};

WIKIDOT.modules.ManageSitePrivateSettingsModule.callbacks = {
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Changes saved";
		w.show();
	}
};

WIKIDOT.modules.ManageSitePrivateSettingsModule.utils = {
	addViewer: function(userId, userName){
		var cont = $("viewers-list-div");
		var vid = "viewer-entry-"+userId;
		
		if(!$(vid)){
			// add user	
			var di = document.createElement('div');
			di.id = vid;
			di.innerHTML = userName;
			cont.appendChild(di);
			WIKIDOT.modules.ManageSitePrivateSettingsModule.utils.updateViewers();
		}
		$("user-lookup").value='';
	},
	
	removeUser: function(userId){
		var cont = $("viewers-list-div");
		var vid = "viewer-entry-"+userId;
		
		if($(vid)){
			cont.removeChild($(vid));
			WIKIDOT.modules.ManageSitePrivateSettingsModule.utils.updateViewers();
		}
	},
	
	updateViewers: function(){
		var dcont = $("extra-viewers-display-list");
		var sr = $("viewers-list-div");
		var ents = sr.getElementsByTagName('div');
		var uss = new Array();
		for(var i=0; i<ents.length;i++){
			var userId = ents[i].id.replace(/.*?([0-9]+)$/,"$1");
			var str = WIKIDOT.render.printuser(userId,ents[i].innerHTML, true);
			str += '(<a href="javascript:;" title="remove from the list" onclick="WIKIDOT.modules.ManageSitePrivateSettingsModule.utils.removeUser('+userId+')">x</a>)';
			uss.push(str);
		}
		if(uss.length == 0){
			dcont.innerHTML = 'No extra access granted.';
		}else{
			dcont.innerHTML = uss.join(', ');
		}
	}
	
};

WIKIDOT.modules.ManageSitePrivateSettingsModule.init = function(){
	// attach the autocomplete thing
	var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
	myDataSource.scriptQueryParam="q";
	myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";

	var myAutoComp = new YAHOO.widget.AutoComplete("sm-private-land","sm-private-land-list", myDataSource);
	myAutoComp.formatResult = function(aResultItem, sQuery) { 
		var title = aResultItem[1];
		var unixName = aResultItem[0];
		if(unixName!= null){
			return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 80%;">('+title+')</div>';
		} else {
			return "";
		}
	}
	
	myAutoComp.autoHighlight = false;
	myAutoComp.minQueryLength = 2;
	myAutoComp.queryDelay = 0.5;
	
	// init autocomplete now
	var dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users','name', 'user_id']); 
	dataSource.scriptQueryParam="q";
	dataSource.scriptQueryAppend = "&module=UserLookupQModule";
	
	var autoComp = new YAHOO.widget.AutoComplete('user-lookup','user-lookup-list', dataSource);

	autoComp.minQueryLength = 2;
	autoComp.queryDelay = 0.5;
	autoComp.forceSelection = true;
	autoComp.itemSelectEvent.subscribe(function(sType, args){
		var userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/,"$1");
		var userName = args[1].getElementsByTagName('div').item(0).innerHTML;
		WIKIDOT.modules.ManageSitePrivateSettingsModule.utils.addViewer(userId, userName);
	});
			
	autoComp.formatResult = function(aResultItem, sQuery) { 
		var name = aResultItem[0];
		var userId = aResultItem[1];
		if(name!= null){
			return '<div id="user-autocomplete-'+userId+'">'+name+'</div>';
		} else {
			return "";
		}
			
	}	
	
	WIKIDOT.modules.ManageSitePrivateSettingsModule.utils.updateViewers();
}

WIKIDOT.modules.ManageSitePrivateSettingsModule.init();
