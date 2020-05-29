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

WIKIDOT.modules.WhoInvitedModule = {}

WIKIDOT.modules.WhoInvitedModule.vars = {
	currentUserId: null
}

WIKIDOT.modules.WhoInvitedModule.listeners = {
	lookUp: function(e){
		var userId = WIKIDOT.modules.WhoInvitedModule.vars.currentUserId;
		if(userId == null){
			alert("No member has been selected yet.");
			YAHOO.util.Event.preventDefault(e);
			return;
		}
		var p = new Object();
		p.userId = userId;
		OZONE.ajax.requestModule("wiki/invitations/WhoInvitedResultsModule", p, WIKIDOT.modules.WhoInvitedModule.callbacks.lookUp);
		YAHOO.util.Event.preventDefault(e);
	}	

}

WIKIDOT.modules.WhoInvitedModule.callbacks = {
	lookUp: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("who-invited-results-box").innerHTML = r.body;
	}

}

WIKIDOT.modules.WhoInvitedModule.init = function(){
	OZONE.dom.onDomReady(function(){
		// init autocomplete now
		var dataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['users','name', 'user_id']); 
		dataSource.scriptQueryParam="q";
		dataSource.scriptQueryAppend = "&module=MemberLookupQModule&siteId="+WIKIREQUEST.info.siteId;
		
		var autoComp = new YAHOO.widget.AutoComplete('user-lookup','user-lookup-list', dataSource);
			
		autoComp.itemSelectEvent.subscribe(function(sType, args){
			var userId = args[1].getElementsByTagName('div').item(0).id.replace(/.*?([0-9]+)$/,"$1");
			var userName = args[1].getElementsByTagName('div').item(0).innerHTML;
			WIKIDOT.modules.WhoInvitedModule.vars.currentUserId = userId;
		});
			
		autoComp.minQueryLength = 2;
		autoComp.queryDelay = 0.5;
		autoComp.forceSelection = true;

		autoComp.formatResult = function(aResultItem, sQuery) { 
			var name = aResultItem[0];
			var userId = aResultItem[1];
			if(name!= null){
				return '<div id="user-autocomplete-'+userId+'">'+name+'</div>';
			} else {
				return "";
			}
				
		}	
	}, "dummy-ondomready-block");
}

WIKIDOT.modules.WhoInvitedModule.init();
